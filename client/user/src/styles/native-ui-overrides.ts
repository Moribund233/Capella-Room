import type { GlobalThemeOverrides } from 'naive-ui'

export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#07C160',
    primaryColorHover: '#06AD56',
    primaryColorPressed: '#05944F',
    primaryColorSuppl: '#07C160',

    textColorBase: '#262626',
    textColor1: '#262626',
    textColor2: '#595959',
    textColor3: '#8C8C8C',

    borderColor: '#D9D9D9',
    dividerColor: '#F0F0F0',

    fontSize: '14px',
    fontSizeMini: '12px',
    fontSizeTiny: '12px',
    fontSizeSmall: '14px',
    fontSizeMedium: '14px',
    fontSizeLarge: '16px',
    fontSizeHuge: '20px',
  },
  Button: {
    borderRadiusSmall: '4px',
    borderRadiusMedium: '8px',
    borderRadiusLarge: '8px',
  },
  Input: {
    borderRadius: '8px',
    heightMedium: '40px',
  },
  Card: {
    borderRadius: '12px',
  },
  Modal: {
    borderRadius: '16px',
  },
}
