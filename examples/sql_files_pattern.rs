//! Демонстрация вынесения SQL-запросов в отдельные файлы

fn main() {
    println!("=== SQL-запросы в отдельных файлах ===\n");

    println!("📁 Структура SQL-файлов:");
    println!();
    println!("  upm-core/src/sql/");
    println!("  ├── schema.sql                    # Схема БД");
    println!("  └── queries/");
    println!("      ├── add_package.sql           # INSERT пакета");
    println!("      ├── get_package_by_name.sql   # SELECT пакета");
    println!("      ├── check_package_exists.sql  # Проверка существования");
    println!("      ├── get_package_status.sql    # Получить статус");
    println!("      ├── delete_package.sql        # DELETE пакета");
    println!("      ├── update_package_status.sql # UPDATE статуса");
    println!("      ├── update_package.sql        # UPDATE всех полей");
    println!("      └── health_check.sql          # Проверка здоровья БД ✨");
    println!();

    println!("🔧 Как это работает:");
    println!();
    println!("  1️⃣  SQL-запрос в отдельном файле:");
    println!();
    println!("      // health_check.sql");
    println!("      SELECT 1 as health_check");
    println!();

    println!("  2️⃣  Загрузка на этапе компиляции:");
    println!();
    println!("      pub async fn pool_is_healthy(&self) -> bool {{");
    println!("          const HEALTH_CHECK_SQL: &str = include_str!(\"../sql/queries/health_check.sql\");");
    println!("          ");
    println!("          sqlx::query(HEALTH_CHECK_SQL)");
    println!("              .fetch_one(&self.pool)");
    println!("              .await");
    println!("              .is_ok()");
    println!("      }}");
    println!();

    println!("💡 Преимущества:");
    println!();
    println!("  ✅ SQL отделён от кода Rust");
    println!("     - Легче читать");
    println!("     - Легче редактировать");
    println!("     - Можно использовать SQL-форматтеры");
    println!();

    println!("  ✅ Проверка на этапе компиляции");
    println!("     - include_str!() встраивает файл в бинарник");
    println!("     - Ошибка компиляции если файл не найден");
    println!("     - Нет runtime overhead");
    println!();

    println!("  ✅ Переиспользование SQL");
    println!("     - Можно использовать один файл в разных местах");
    println!("     - Централизованное управление запросами");
    println!();

    println!("  ✅ Версионирование");
    println!("     - SQL-файлы в git");
    println!("     - История изменений");
    println!("     - Code review SQL-запросов");
    println!();

    println!("📖 Примеры других функций:");
    println!();

    println!("  🔹 add_package():");
    println!(
        "      const ADD_PACKAGE_SQL: &str = include_str!(\"../sql/queries/add_package.sql\");"
    );
    println!("      package.bind_to_insert_query(sqlx::query(ADD_PACKAGE_SQL))");
    println!();

    println!("  🔹 get_package_from_database_by_name():");
    println!("      const GET_PACKAGE_SQL: &str = include_str!(\"../sql/queries/get_package_by_name.sql\");");
    println!("      sqlx::query_as::<_, Package>(GET_PACKAGE_SQL).bind(package_name)");
    println!();

    println!("  🔹 update_package_status_in_database():");
    println!("      const UPDATE_STATUS_SQL: &str = include_str!(\"../sql/queries/update_package_status.sql\");");
    println!("      sqlx::query(UPDATE_STATUS_SQL).bind(new_status).bind(package_name)");
    println!();

    println!("  🔹 pool_is_healthy() (новая!):");
    println!(
        "      const HEALTH_CHECK_SQL: &str = include_str!(\"../sql/queries/health_check.sql\");"
    );
    println!("      sqlx::query(HEALTH_CHECK_SQL).fetch_one(&self.pool)");
    println!();

    println!("🎯 Паттерн использования:");
    println!();
    println!("  1. Создать SQL-файл в src/sql/queries/");
    println!("  2. Написать SQL-запрос");
    println!("  3. Загрузить через include_str!()");
    println!("  4. Использовать с sqlx::query()");
    println!();

    println!("📝 Содержимое health_check.sql:");
    println!();
    println!("  SELECT 1 as health_check");
    println!();
    println!("  Простой запрос для проверки:");
    println!("  - Доступна ли БД");
    println!("  - Отвечает ли пул");
    println!("  - Можно ли выполнять запросы");
    println!();

    println!("🔄 Использование в ensure_connection():");
    println!();
    println!("  pub async fn ensure_connection(&mut self) -> Result<bool, DataBaseError> {{");
    println!("      // Проверяем здоровье пула (использует health_check.sql)");
    println!("      if self.pool_is_healthy().await {{");
    println!("          return Ok(true);");
    println!("      }}");
    println!();
    println!("      // Пул не отвечает, пересоздаём");
    println!("      self.recreate_pool().await?;");
    println!("      ");
    println!("      // Проверяем, что новый пул работает");
    println!("      if !self.pool_is_healthy().await {{");
    println!("          return Err(DataBaseError::ConnectionError(");
    println!("              \"Failed to recreate pool\".to_string()");
    println!("          ));");
    println!("      }}");
    println!();
    println!("      Ok(false)");
    println!("  }}");
    println!();

    println!("=== Готово! ===");
}
