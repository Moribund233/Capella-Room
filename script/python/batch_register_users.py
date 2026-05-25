"""
批量注册测试用户脚本

使用方式:
    python batch_register_users.py [数量] [起始编号] [端口]
    
示例:
    python batch_register_users.py              # 注册10个用户，从1开始，端口8765
    python batch_register_users.py 20           # 注册20个用户，从1开始，端口8765
    python batch_register_users.py 20 1         # 注册20个用户，从1开始，端口8765
    python batch_register_users.py 20 1 8080    # 注册20个用户，从1开始，端口8080
    
环境变量:
    TEST_PORT: 服务器端口 (默认8765)

注册格式: TestUser{n}@test.com (n从1开始，跳过已存在的)
默认密码: Test12345
"""

import requests
import json
import time
import sys
import os
from typing import List, Dict

# 配置
DEFAULT_PORT = int(os.getenv('TEST_PORT', '8765'))
DEFAULT_PASSWORD = "Test12345"


def get_base_url(port: int) -> str:
    """获取基础URL"""
    return f"http://localhost:{port}"


def register_user(base_url: str, email: str, password: str, username: str) -> Dict:
    """
    注册单个用户
    
    Args:
        base_url: 基础URL
        email: 邮箱
        password: 密码
        username: 用户名
        
    Returns:
        注册结果
    """
    url = f"{base_url}/api/v1/auth/register"
    data = {
        'email': email,
        'password': password,
        'username': username
    }
    
    try:
        response = requests.post(url, json=data, timeout=10)
        return response.json()
    except Exception as e:
        return {'success': False, 'message': str(e)}


def batch_register_users(base_url: str, start_num: int = 1, count: int = 10, skip_list: List[int] = None) -> Dict:
    """
    批量注册测试用户
    
    Args:
        base_url: 基础URL
        start_num: 起始编号
        count: 注册数量
        skip_list: 需要跳过的编号列表
        
    Returns:
        注册统计结果
    """
    if skip_list is None:
        skip_list = []
    
    results = {
        'total_attempted': 0,
        'success': 0,
        'failed': 0,
        'skipped': 0,
        'successful_users': [],
        'failed_details': []
    }
    
    print("=" * 60)
    print("开始批量注册测试用户")
    print("=" * 60)
    print(f"目标数量: {count} 个用户")
    print(f"跳过编号: {skip_list}")
    print(f"基础URL: {base_url}")
    print("=" * 60)
    
    current_num = start_num
    registered_count = 0
    
    while registered_count < count:
        # 检查是否需要跳过
        if current_num in skip_list:
            print(f"[跳过] TestUser{current_num}@test.com (在跳过列表中)")
            results['skipped'] += 1
            current_num += 1
            continue
        
        email = f"TestUser{current_num}@test.com"
        username = f"TestUser{current_num}"
        
        print(f"\n[{registered_count + 1}/{count}] 正在注册: {email} ...", end=" ")
        
        result = register_user(base_url, email, DEFAULT_PASSWORD, username)
        results['total_attempted'] += 1
        
        if result.get('success'):
            print("✅ 成功")
            results['success'] += 1
            results['successful_users'].append({
                'num': current_num,
                'email': email,
                'username': username,
                'user_id': result.get('data', {}).get('id')
            })
            registered_count += 1
        else:
            error_msg = result.get('message', '未知错误')
            # 检查是否是已存在的用户
            if '已被注册' in error_msg or '已存在' in error_msg or 'already exists' in error_msg.lower():
                print(f"⚠️ 已存在，跳过")
                results['skipped'] += 1
            else:
                print(f"❌ 失败: {error_msg}")
                results['failed'] += 1
                results['failed_details'].append({
                    'num': current_num,
                    'email': email,
                    'error': error_msg
                })
                registered_count += 1  # 失败也计入进度，避免无限循环
        
        current_num += 1
        
        # 添加短暂延迟，避免请求过快
        time.sleep(0.02)
    
    return results


def print_summary(results: Dict):
    """打印注册结果摘要"""
    print("\n" + "=" * 60)
    print("注册结果摘要")
    print("=" * 60)
    print(f"尝试注册: {results['total_attempted']} 个")
    print(f"成功: {results['success']} 个")
    print(f"失败: {results['failed']} 个")
    print(f"跳过(已存在): {results['skipped']} 个")
    
    if results['successful_users']:
        print("\n成功注册的用户:")
        for user in results['successful_users']:
            print(f"  ✅ {user['email']} (ID: {user['user_id']})")
    
    if results['failed_details']:
        print("\n注册失败的用户:")
        for detail in results['failed_details']:
            print(f"  ❌ TestUser{detail['num']}@test.com - {detail['error']}")
    
    print("\n" + "=" * 60)
    print("用户登录信息:")
    print("=" * 60)
    print(f"邮箱格式: TestUser{{n}}@test.com")
    print(f"统一密码: {DEFAULT_PASSWORD}")
    print("=" * 60)


def main():
    """主函数"""
    # 默认参数
    start_num = 1
    count = 10
    port = DEFAULT_PORT
    skip_list = []  # 不跳过任何用户，已存在的用户会在注册时自动跳过
    
    # 解析命令行参数
    if len(sys.argv) > 1:
        try:
            count = int(sys.argv[1])
        except ValueError:
            pass
    
    if len(sys.argv) > 2:
        try:
            start_num = int(sys.argv[2])
        except ValueError:
            pass
    
    if len(sys.argv) > 3:
        try:
            port = int(sys.argv[3])
        except ValueError:
            pass
    
    base_url = get_base_url(port)
    
    print("批量注册测试用户脚本")
    print(f"用法: python batch_register_users.py [数量] [起始编号] [端口]")
    print(f"示例: python batch_register_users.py 20 1 8080  # 注册20个用户，从TestUser1开始，端口8080")
    print(f"当前配置: 数量={count}, 起始编号={start_num}, 端口={port}")
    print()
    
    # 检查服务器连接
    try:
        response = requests.get(f"{base_url}/health", timeout=5)
        print(f"服务器健康检查: {'正常' if response.status_code == 200 else '异常'}")
    except Exception as e:
        print(f"⚠️ 无法连接到服务器: {e}")
        print("请确保服务器已启动后再运行脚本")
        return
    
    # 执行批量注册
    results = batch_register_users(base_url, start_num=start_num, count=count, skip_list=skip_list)
    
    # 打印结果
    print_summary(results)


if __name__ == "__main__":
    main()
