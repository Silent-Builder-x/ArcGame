use arcis::*;

#[encrypted]
mod battle_engine {
    use arcis::*;

    pub struct PlayerMove {
        pub action_type: u64, // 1=攻击, 2=防御, 3=破防
        pub power: u64,       // 力量值
    }

    pub struct BattleResult {
        pub winner: u64,      // 1=玩家1胜, 2=玩家2胜, 0=平局
        pub damage: u64,      // 造成的伤害值
    }

    #[instruction]
    pub fn resolve_duel(
        move_a_ctxt: Enc<Shared, PlayerMove>,
        move_b_ctxt: Enc<Shared, PlayerMove>
    ) -> Enc<Shared, BattleResult> {
        let a = move_a_ctxt.to_arcis();
        let b = move_b_ctxt.to_arcis();

        // --- 核心博弈逻辑 (石头剪刀布变体) ---
        // 1(攻击) > 3(破防)
        // 3(破防) > 2(防御)
        // 2(防御) > 1(攻击)

        // 判定 A 是否因属性克制获胜
        let a_type_wins = 
            (a.action_type == 1 && b.action_type == 3) ||
            (a.action_type == 3 && b.action_type == 2) ||
            (a.action_type == 2 && b.action_type == 1);

        // 判定 B 是否因属性克制获胜
        let b_type_wins = 
            (b.action_type == 1 && a.action_type == 3) ||
            (b.action_type == 3 && a.action_type == 2) ||
            (b.action_type == 2 && a.action_type == 1);

        // 判定是否同属性
        let same_type = a.action_type == b.action_type;

        // 同属性时比较力量值
        let a_power_wins = a.power > b.power;
        let b_power_wins = b.power > a.power;

        // --- 最终胜负选择器 ---
        let (winner, raw_dmg) = if a_type_wins {
            (1u64, a.power) // 克制直接造成全额伤害
        } else {
            if b_type_wins {
                (2u64, b.power)
            } else {
                // 同属性，拼点数
                if same_type {
                    if a_power_wins {
                        (1u64, a.power - b.power) // 拼点，伤害抵消
                    } else {
                        if b_power_wins {
                            (2u64, b.power - a.power)
                        } else {
                            (0u64, 0u64) // 完全平局
                        }
                    }
                } else {
                    (0u64, 0u64) // 理论上不可达，但也设为平局
                }
            }
        };

        let result = BattleResult {
            winner,
            damage: raw_dmg,
        };

        // 结果加密返回给 Solana 上的游戏裁判程序
        move_a_ctxt.owner.from_arcis(result)
    }
}