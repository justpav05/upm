//! Демонстрация автоматической инициализации схемы БД

fn main() {
    println!("=== Автоматическая инициализация схемы БД ===\n");

    println!("🔧 Как это работает:");
    println!();

    println!("  📝 БЫЛО (нужно было вызывать отдельно):");
    println!();
    println!("      let db = DataBase::new(path, \"db.db\".to_string(), 5).await?;");
    println!("      ");
    println!("      // ❌ Нужно вручную инициализировать схему");
    println!("      db.init_schema(Path::new(\"schema.sql\")).await?;");
    println!();

    println!("  ✅ СТАЛО (автоматически):");
    println!();
    println!("      let db = DataBase::new(path, \"db.db\".to_string(), 5).await?;");
    println!("      ");
    println!("      // ✅ Схема уже инициализирована!");
    println!("      // Можно сразу работать с БД");
    println!("      db.add_package(&package).await?;");
    println!();

    println!("📋 Что происходит внутри new():");
    println!();
    println!("  1️⃣  Проверка прав (root на Unix)");
    println!("  2️⃣  Проверка существования пути");
    println!("  3️⃣  Создание пути к БД");
    println!("  4️⃣  Создание пула подключений");
    println!("  5️⃣  📦 Автоматическая инициализация схемы ← НОВОЕ!");
    println!("  6️⃣  Возврат готовой БД");
    println!();

    println!("💻 Код внутри new():");
    println!();
    println!("      // Создаём пул");
    println!("      let pool = SqlitePoolOptions::new()");
    println!("          .max_connections(max_connections)");
    println!("          .connect_with(connect_options)");
    println!("          .await?;");
    println!();
    println!("      // Создаём структуру");
    println!("      let db = Self {{");
    println!("          pool,");
    println!("          database_path,");
    println!("          max_connections,");
    println!("      }};");
    println!();
    println!("      // ✨ Автоматически инициализируем схему из SQL-файла");
    println!("      const SCHEMA_SQL: &str = include_str!(\"../sql/schema.sql\");");
    println!("      sqlx::query(SCHEMA_SQL).execute(&db.pool).await?;");
    println!();
    println!("      // Возвращаем готовую БД");
    println!("      Ok(db)");
    println!();

    println!("📁 SQL-файл схемы:");
    println!();
    println!("  Файл: upm-core/src/sql/schema.sql");
    println!();
    println!("  Содержит:");
    println!("  - CREATE TABLE IF NOT EXISTS packages (...)");
    println!("  - CREATE INDEX IF NOT EXISTS ...");
    println!("  - CREATE TABLE IF NOT EXISTS dependencies (...)");
    println!("  - И другие таблицы");
    println!();

    println!("✅ Преимущества:");
    println!();
    println!("  1. Не нужно вызывать init_schema() отдельно");
    println!("     - Меньше кода");
    println!("     - Невозможно забыть инициализировать");
    println!();

    println!("  2. БД всегда готова к работе");
    println!("     - После new() можно сразу использовать");
    println!("     - Гарантированно корректная схема");
    println!();

    println!("  3. Схема в SQL-файле");
    println!("     - Легко редактировать");
    println!("     - Версионируется в git");
    println!("     - Проверяется на этапе компиляции (include_str!)");
    println!();

    println!("  4. CREATE TABLE IF NOT EXISTS");
    println!("     - Безопасно вызывать несколько раз");
    println!("     - Не упадёт если таблица уже существует");
    println!();

    println!("🔄 Пересоздание пула:");
    println!();
    println!("  При вызове recreate_pool() схема НЕ пересоздаётся:");
    println!();
    println!("      db.recreate_pool().await?;");
    println!();
    println!("  Потому что:");
    println!("  - Схема уже существует в БД");
    println!("  - CREATE TABLE IF NOT EXISTS безопасно");
    println!("  - Пересоздаётся только пул подключений");
    println!();

    println!("📖 Пример использования:");
    println!();
    println!("  use std::path::Path;");
    println!("  use upm_core::core::database::DataBase;");
    println!();
    println!("  #[tokio::main]");
    println!("  async fn main() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("      // Создаём БД (схема инициализируется автоматически!)");
    println!("      let db = DataBase::new(");
    println!("          Path::new(\"/var/lib/upm\"),");
    println!("          \"packages.db\".to_string(),");
    println!("          5");
    println!("      ).await?;");
    println!();
    println!("      // ✅ Можем сразу работать!");
    println!("      let package = Package {{ /* ... */ }};");
    println!("      db.add_package(&package).await?;");
    println!();
    println!("      Ok(())");
    println!("  }}");
    println!();

    println!("⚠️  Важно:");
    println!();
    println!("  - Схема инициализируется ОДИН РАЗ при создании БД");
    println!("  - Используется CREATE TABLE IF NOT EXISTS");
    println!("  - Безопасно для существующих БД");
    println!("  - SQL-файл встраивается на этапе компиляции");
    println!();

    println!("🎯 Итог:");
    println!();
    println!("  Теперь DataBase::new() делает ВСЁ:");
    println!("  ✅ Проверяет права");
    println!("  ✅ Создаёт подключение");
    println!("  ✅ Инициализирует схему");
    println!("  ✅ Возвращает готовую БД");
    println!();
    println!("  Один вызов - полностью готовая база данных!");
    println!();

    println!("=== Готово! ===");
}
