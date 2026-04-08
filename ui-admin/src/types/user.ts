/**
 * 用户模块类型定义
 */

/** 更新用户名请求 */
export interface UpdateUsernameRequest {
  username: string
}

/** 修改密码请求 */
export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}
