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

export const darkThemeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#10b981',
    primaryColorHover: '#34d399',
    primaryColorPressed: '#059669',
    primaryColorSuppl: '#10b981',

    textColorBase: 'rgba(255, 255, 255, 0.9)',
    textColor1: 'rgba(255, 255, 255, 0.9)',
    textColor2: 'rgba(255, 255, 255, 0.65)',
    textColor3: 'rgba(255, 255, 255, 0.45)',

    borderColor: 'rgba(255, 255, 255, 0.1)',
    dividerColor: 'rgba(255, 255, 255, 0.06)',

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
    color: 'rgba(31, 31, 31, 1)',
    borderColor: 'rgba(255, 255, 255, 0.1)',
  },
  Modal: {
    borderRadius: '16px',
  },
  List: {
    color: 'transparent',
    borderColor: 'rgba(255, 255, 255, 0.1)',
  },
}
