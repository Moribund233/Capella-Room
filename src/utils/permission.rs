use crate::models::user::UserRole;

pub fn is_admin(role: &UserRole) -> bool {
    role.is_admin()
}

pub fn is_super_admin(role: &UserRole) -> bool {
    role.is_super_admin()
}

pub fn can_manage_user(manager_role: &UserRole, target_role: &UserRole) -> bool {
    manager_role.can_manage_user(target_role)
}

pub fn check_permission(role: &UserRole, required_role: &UserRole) -> bool {
    match required_role {
        UserRole::SuperAdmin => role.is_super_admin(),
        UserRole::Admin => role.is_admin(),
        UserRole::User => true,
    }
}

pub fn role_to_string(role: &UserRole) -> &'static str {
    match role {
        UserRole::SuperAdmin => "super_admin",
        UserRole::Admin => "admin",
        UserRole::User => "user",
    }
}

pub fn string_to_role(s: &str) -> Option<UserRole> {
    match s {
        "super_admin" => Some(UserRole::SuperAdmin),
        "admin" => Some(UserRole::Admin),
        "user" => Some(UserRole::User),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin() {
        assert!(is_admin(&UserRole::Admin));
        assert!(is_admin(&UserRole::SuperAdmin));
        assert!(!is_admin(&UserRole::User));
    }

    #[test]
    fn test_is_super_admin() {
        assert!(is_super_admin(&UserRole::SuperAdmin));
        assert!(!is_super_admin(&UserRole::Admin));
        assert!(!is_super_admin(&UserRole::User));
    }

    #[test]
    fn test_can_manage_user() {
        assert!(can_manage_user(&UserRole::SuperAdmin, &UserRole::Admin));
        assert!(can_manage_user(&UserRole::SuperAdmin, &UserRole::User));
        assert!(can_manage_user(&UserRole::SuperAdmin, &UserRole::SuperAdmin));

        assert!(can_manage_user(&UserRole::Admin, &UserRole::User));
        assert!(!can_manage_user(&UserRole::Admin, &UserRole::Admin));
        assert!(!can_manage_user(&UserRole::Admin, &UserRole::SuperAdmin));

        assert!(!can_manage_user(&UserRole::User, &UserRole::User));
        assert!(!can_manage_user(&UserRole::User, &UserRole::Admin));
        assert!(!can_manage_user(&UserRole::User, &UserRole::SuperAdmin));
    }

    #[test]
    fn test_check_permission() {
        assert!(check_permission(&UserRole::SuperAdmin, &UserRole::User));
        assert!(check_permission(&UserRole::SuperAdmin, &UserRole::Admin));
        assert!(check_permission(&UserRole::SuperAdmin, &UserRole::SuperAdmin));

        assert!(check_permission(&UserRole::Admin, &UserRole::User));
        assert!(check_permission(&UserRole::Admin, &UserRole::Admin));
        assert!(!check_permission(&UserRole::Admin, &UserRole::SuperAdmin));

        assert!(check_permission(&UserRole::User, &UserRole::User));
        assert!(!check_permission(&UserRole::User, &UserRole::Admin));
        assert!(!check_permission(&UserRole::User, &UserRole::SuperAdmin));
    }

    #[test]
    fn test_role_string_conversion() {
        assert_eq!(role_to_string(&UserRole::SuperAdmin), "super_admin");
        assert_eq!(role_to_string(&UserRole::Admin), "admin");
        assert_eq!(role_to_string(&UserRole::User), "user");

        assert_eq!(string_to_role("super_admin"), Some(UserRole::SuperAdmin));
        assert_eq!(string_to_role("admin"), Some(UserRole::Admin));
        assert_eq!(string_to_role("user"), Some(UserRole::User));
        assert_eq!(string_to_role("invalid"), None);
    }
}
