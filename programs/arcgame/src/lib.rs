use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_GAME: u32 = comp_def_offset("resolve_duel");

declare_id!("8C9qag5tsr6xiqhK8PSj7mh1pDYKbRSp7tP4MLfhVdjc");

#[arcium_program]
pub mod arcgame {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [新增] 创建对局
    pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.player_1 = ctx.accounts.player.key();
        game.turn = 1;
        game.state = 0; // 0: Waiting for P2 Join/Move
        msg!("Game lobby created by {}", game.player_1);
        Ok(())
    }

    /// [新增] 加入游戏 (Player 2)
    pub fn join_game(ctx: Context<JoinGame>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.player_2 == Pubkey::default(), GameError::GameFull);
        game.player_2 = ctx.accounts.player.key();
        msg!("Player 2 joined: {}", game.player_2);
        Ok(())
    }

    /// [核心] 提交加密动作
    /// 玩家提交他们在本地加密好的 Move (Type + Power)
    /// 这些数据以密文形式存储在链上，对手不可见
    pub fn submit_move(
        ctx: Context<SubmitMove>,
        encrypted_type: [u8; 32],
        encrypted_power: [u8; 32],
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let signer = ctx.accounts.player.key();

        if signer == game.player_1 {
            game.p1_move_type = encrypted_type;
            game.p1_move_power = encrypted_power;
            game.p1_committed = true;
        } else if signer == game.player_2 {
            game.p2_move_type = encrypted_type;
            game.p2_move_power = encrypted_power;
            game.p2_committed = true;
        } else {
            return Err(GameError::NotAPlayer.into());
        }

        msg!("Player {} submitted a hidden move.", signer);
        Ok(())
    }

    /// [升级] 结算回合
    /// 只有当两名玩家都提交了动作后才能调用
    pub fn resolve_turn(
        ctx: Context<ResolveTurn>,
        computation_offset: u64,
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let game = &ctx.accounts.game;
        require!(game.p1_committed && game.p2_committed, GameError::WaitingForMoves);

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        // 构建 MPC 参数: P1 Move + P2 Move
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            // P1
            .encrypted_u64(game.p1_move_type)
            .encrypted_u64(game.p1_move_power)
            // P2
            .encrypted_u64(game.p2_move_type)
            .encrypted_u64(game.p2_move_power)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![ResolveDuelCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "resolve_duel")]
    pub fn resolve_duel_callback(
        ctx: Context<ResolveDuelCallback>,
        output: SignedComputationOutputs<ResolveDuelOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(ResolveDuelOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // 解析结果
        let winner_bytes: [u8; 8] = o.ciphertexts[0][0..8].try_into().unwrap();
        let dmg_bytes: [u8; 8] = o.ciphertexts[1][0..8].try_into().unwrap();

        let winner = u64::from_le_bytes(winner_bytes);
        let dmg = u64::from_le_bytes(dmg_bytes);

        // 重置回合状态
        let game = &mut ctx.accounts.game;
        game.p1_committed = false;
        game.p2_committed = false;
        game.turn += 1;

        msg!("⚔️ Round Resolved via MPC!");
        msg!("Winner: Player {}, Damage: {}", winner, dmg);
        
        emit!(RoundEndEvent {
            game: game.key(),
            winner_id: winner as u8,
            damage: dmg,
            turn: game.turn - 1,
        });
        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(
        init, 
        payer = player, 
        space = 8 + 32 + 32 + 1 + 1 + 1 + 1 + (32*4) + 100, 
        seeds = [b"game", player.key().as_ref()],
        bump
    )]
    pub game: Account<'info, GameState>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, GameState>,
    #[account(mut)]
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitMove<'info> {
    #[account(mut)]
    pub game: Account<'info, GameState>,
    #[account(mut)]
    pub player: Signer<'info>,
}

#[account]
pub struct GameState {
    pub player_1: Pubkey,
    pub player_2: Pubkey,
    
    // P1 Move Storage (Encrypted)
    pub p1_move_type: [u8; 32],
    pub p1_move_power: [u8; 32],
    pub p1_committed: bool,

    // P2 Move Storage (Encrypted)
    pub p2_move_type: [u8; 32],
    pub p2_move_power: [u8; 32],
    pub p2_committed: bool,

    pub turn: u64,
    pub state: u8,
}

#[queue_computation_accounts("resolve_duel", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ResolveTurn<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, GameState>,
    
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GAME))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("resolve_duel")]
#[derive(Accounts)]
pub struct ResolveDuelCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_GAME))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub game: Account<'info, GameState>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("resolve_duel", payer)]
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: LUT
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT Prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct RoundEndEvent {
    pub game: Pubkey,
    pub winner_id: u8,
    pub damage: u64,
    pub turn: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}

#[error_code]
pub enum GameError {
    #[msg("Game is full")] GameFull,
    #[msg("Not a player in this game")] NotAPlayer,
    #[msg("Waiting for other player to move")] WaitingForMoves,
}