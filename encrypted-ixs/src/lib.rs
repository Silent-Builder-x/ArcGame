use arcis::*;

#[encrypted]
mod card_battle_engine {
    use arcis::*;

    pub struct GameState {
        pub player_a_card: u64, // 加密的手牌 A
        pub player_b_card: u64, // 加密的手牌 B
    }

    pub struct RoundResult {
        pub winner_id: u64,    // 1 = A 赢, 2 = B 赢, 0 = 平局
        pub damage_dealt: u64, // 基于数值差额计算的机密伤害
    }

    #[instruction]
    pub fn resolve_round(
        state_ctxt: Enc<Shared, GameState>
    ) -> Enc<Shared, RoundResult> {
        let state = state_ctxt.to_arcis();
        
        let a_val = state.player_a_card;
        let b_val = state.player_b_card;

        // 执行同态比较逻辑
        let a_wins = a_val > b_val;
        let b_wins = b_val > a_val;

        // 使用 V4 规范的 if-else Mux 链判定胜者
        let (winner, damage) = if a_wins {
            (1u64, a_val - b_val)
        } else {
            if b_wins {
                (2u64, b_val - a_val)
            } else {
                (0u64, 0u64) // 平局
            }
        };

        let result = RoundResult {
            winner_id: winner,
            damage_dealt: damage,
        };

        state_ctxt.owner.from_arcis(result)
    }
}