//! 创建初始管理员用户
//!
//! 用法:
//!   cargo run --bin create-admin -- --username <用户名> --password <密码> [--email <邮箱>] [--display-name <显示名称>]
//!
//! 环境变量:
//!   DATABASE_URL - SQLite 数据库路径
//!
//! 示例:
//!   cargo run --bin create-admin -- --username admin --password admin123
//!   cargo run --bin create-admin -- -u admin -p admin123 -e admin@example.com -d "系统管理员"

use std::env;
use std::io::{self, Write};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

/// 命令行参数
struct Args {
    username: String,
    password: String,
    email: Option<String>,
    display_name: Option<String>,
    database_url: String,
}

fn print_usage() {
    eprintln!(
        r#"
创建初始管理员用户

用法:
  create-admin --username <用户名> --password <密码> [选项]
  create-admin -u <用户名> -p <密码> [选项]

选项:
  -u, --username <用户名>       管理员用户名 (必填)
  -p, --password <密码>         管理员密码 (必填，至少 6 位)
  -e, --email <邮箱>            管理员邮箱 (可选)
  -d, --display-name <名称>     显示名称 (可选)
  --database-url <URL>          数据库路径 (可选，默认从 DATABASE_URL 环境变量读取)
  -h, --help                    显示此帮助信息

环境变量:
  DATABASE_URL                  SQLite 数据库路径，例如 sqlite:./aicodex.db

示例:
  create-admin -u admin -p admin123
  create-admin -u admin -p admin123 -e admin@example.com -d "系统管理员"
  DATABASE_URL=sqlite:./aicodex.db create-admin -u admin -p admin123
"#
    );
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("参数不足".to_string());
    }

    let mut username = None;
    let mut password = None;
    let mut email = None;
    let mut display_name = None;
    let mut database_url = env::var("DATABASE_URL").ok();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-u" | "--username" => {
                i += 1;
                if i >= args.len() {
                    return Err("--username 需要参数".to_string());
                }
                username = Some(args[i].clone());
            }
            "-p" | "--password" => {
                i += 1;
                if i >= args.len() {
                    return Err("--password 需要参数".to_string());
                }
                password = Some(args[i].clone());
            }
            "-e" | "--email" => {
                i += 1;
                if i >= args.len() {
                    return Err("--email 需要参数".to_string());
                }
                email = Some(args[i].clone());
            }
            "-d" | "--display-name" => {
                i += 1;
                if i >= args.len() {
                    return Err("--display-name 需要参数".to_string());
                }
                display_name = Some(args[i].clone());
            }
            "--database-url" => {
                i += 1;
                if i >= args.len() {
                    return Err("--database-url 需要参数".to_string());
                }
                database_url = Some(args[i].clone());
            }
            arg => {
                return Err(format!("未知参数: {}", arg));
            }
        }
        i += 1;
    }

    let username = username.ok_or("缺少必填参数: --username")?;
    let password = password.ok_or("缺少必填参数: --password")?;
    let database_url = database_url.ok_or("缺少数据库路径，请设置 DATABASE_URL 环境变量或使用 --database-url 参数")?;

    // 验证
    if username.len() < 3 {
        return Err("用户名长度至少 3 个字符".to_string());
    }
    if password.len() < 6 {
        return Err("密码长度至少 6 个字符".to_string());
    }

    Ok(Args {
        username,
        password,
        email,
        display_name,
        database_url,
    })
}

/// 哈希密码
fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("密码哈希失败: {}", e))
}

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenvy::dotenv().ok();

    // 解析参数
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("错误: {}", e);
            eprintln!();
            print_usage();
            std::process::exit(1);
        }
    };

    println!("正在连接数据库...");

    // 连接数据库
    let pool = match SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&args.database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("数据库连接失败: {}", e);
            eprintln!("请确保数据库文件存在且路径正确");
            std::process::exit(1);
        }
    };

    // 检查用户名是否已存在
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = ?")
        .bind(&args.username)
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    if exists.0 > 0 {
        eprintln!("错误: 用户名 '{}' 已存在", args.username);
        std::process::exit(1);
    }

    // 如果提供了邮箱，检查邮箱是否已存在
    if let Some(ref email) = args.email {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap_or((0,));

        if exists.0 > 0 {
            eprintln!("错误: 邮箱 '{}' 已被使用", email);
            std::process::exit(1);
        }
    }

    // 哈希密码
    print!("正在创建管理员用户...");
    io::stdout().flush().ok();

    let password_hash = match hash_password(&args.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("\n{}", e);
            std::process::exit(1);
        }
    };

    // 生成用户 ID
    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // 创建管理员用户
    let result = sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name, role, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'admin', 1, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&args.username)
    .bind(&args.email)
    .bind(&password_hash)
    .bind(&args.display_name)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            println!(" 完成!");
            println!();
            println!("管理员用户创建成功:");
            println!("  用户 ID:    {}", user_id);
            println!("  用户名:     {}", args.username);
            if let Some(email) = &args.email {
                println!("  邮箱:       {}", email);
            }
            if let Some(display_name) = &args.display_name {
                println!("  显示名称:   {}", display_name);
            }
            println!("  角色:       admin");
            println!();
            println!("您现在可以使用此账号登录系统。");
        }
        Err(e) => {
            eprintln!("\n创建管理员用户失败: {}", e);
            std::process::exit(1);
        }
    }
}
