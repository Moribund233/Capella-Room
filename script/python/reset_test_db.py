"""
重置测试数据库脚本
从 .env.test 读取数据库配置，删除并重建测试数据库。
用法: python script/reset_test_db.py
"""

import re
import subprocess
import sys
from pathlib import Path


def load_env_test() -> dict:
    """解析 .env.test 文件，提取环境变量"""
    env_file = Path(__file__).resolve().parent.parent / ".env.test"
    if not env_file.exists():
        print(f"错误: 找不到 {env_file}")
        sys.exit(1)

    config = {}
    with open(env_file, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, value = line.split("=", 1)
                config[key.strip()] = value.strip()
    return config


def parse_pg_url(url: str) -> dict:
    """解析 PostgreSQL URL 为组件"""
    pattern = r"postgres(?:ql)?://(?P<user>[^:]+):(?P<password>[^@]+)@(?P<host>[^:/]+)(?::(?P<port>\d+))?/(?P<dbname>.+)"
    match = re.match(pattern, url)
    if not match:
        print(f"错误: 无法解析 DATABASE_URL: {url}")
        sys.exit(1)
    parts = match.groupdict()
    parts["port"] = parts["port"] or "5432"
    return parts


def main():
    config = load_env_test()
    database_url = config.get("DATABASE_URL")
    if not database_url:
        print("错误: .env.test 中未设置 DATABASE_URL")
        sys.exit(1)

    pg = parse_pg_url(database_url)
    dbname = pg["dbname"]

    # 先连接到 postgres 默认数据库来执行 drop/create
    admin_url = (
        f"postgresql://{pg['user']}:{pg['password']}@{pg['host']}:{pg['port']}/postgres"
    )

    print(f"数据库: {pg['host']}:{pg['port']}/{dbname}")
    print(f"用户:   {pg['user']}")
    print()

    # 终止所有连接到目标数据库的连接
    print(f"[1/3] 终止所有连接到 '{dbname}' 的连接...")
    try:
        subprocess.run(
            [
                "psql",
                admin_url,
                "-c",
                f"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{dbname}' AND pid <> pg_backend_pid();",
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        print("      完成")
    except subprocess.CalledProcessError as e:
        print(f"      注意: {e.stderr.strip() or '无活跃连接'}")

    # 删除数据库
    print(f"[2/3] 删除数据库 '{dbname}'...")
    try:
        subprocess.run(
            ["psql", admin_url, "-c", f"DROP DATABASE IF EXISTS {dbname};"],
            check=True,
            capture_output=True,
            text=True,
        )
        print("      完成")
    except subprocess.CalledProcessError as e:
        print(f"错误: 删除数据库失败: {e.stderr}")
        sys.exit(1)

    # 创建数据库
    print(f"[3/3] 创建数据库 '{dbname}'...")
    try:
        subprocess.run(
            ["psql", admin_url, "-c", f"CREATE DATABASE {dbname};"],
            check=True,
            capture_output=True,
            text=True,
        )
        print("      完成")
    except subprocess.CalledProcessError as e:
        print(f"错误: 创建数据库失败: {e.stderr}")
        sys.exit(1)

    print()
    print("测试数据库已重置成功!")
    print(f"运行测试前确保执行: cargo test")


if __name__ == "__main__":
    main()
