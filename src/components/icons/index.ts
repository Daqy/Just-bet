import GameMinesweeperIcon from '~components/icons/GameMinesweeperIcon.vue'
import GameCoinIcon from '~components/icons/GameCoinIcon.vue'
import GameMouseClickIcon from '~components/icons/GameMouseClickIcon.vue'
import GamePlayerIcon from '~components/icons/GamePlayerIcon.vue'
import AppCopyIcon from '~components/icons/AppCopyIcon.vue'
import AppTickIcon from '~components/icons/AppTickIcon.vue'

export const Icons = {
  coin: GameCoinIcon,
  minesweeper: GameMinesweeperIcon,
  mouseclick: GameMouseClickIcon,
  login: GamePlayerIcon,
  register: GamePlayerIcon,
  copy: {
    icon: AppCopyIcon,
    copied: AppTickIcon,
    bomb: {
      lost: '🟥',
      won: '🟧'
    },
    click: '🟩',
    default: '⬛'
  }
}
