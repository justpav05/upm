//! Демонстрация: все SQL-запросы в отдельных файлах

fn main() {
    println!("=== Все SQL-запросы вынесены в отдельные файлы ===\n");

    println!("📁 Структура SQL-файлов:");
    println!();
    println!("  upm-core/src/sql/");
    println!("  ├── schema.sql                         # Схема БД (автоматически в new())");
    println!("  └── queries/");
    println!("      ├── add_package.sql                # INSERT");
    println!("      ├── get_package_by_name.sql        # SELECT");
    println!("      ├── check_package_exists.sql       # Проверка существования");
    println!("      ├── get_package_status.sql         # Получить статус");
    println!("      ├── delete_package.sql             # DELETE");
    println!("      ├── update_package_status.sql      # UPDATE статуса");
    println!("      ├── update_package.sql             # UPDATE всех полей");
    println!("      ├── health_check.sql               # Проверка здоровья");
    println!("      ├── update_package_name.sql        # UPDATE name ✨");
    println!("      ├── update_package_version.sql     # UPDATE version ✨");
    println!("      ├── update_package_repository.sql  # UPDATE repository ✨");
    println!("      ├── update_package_installed.sql   # UPDATE installed ✨");
    println!("      ├── update_package_description.sql # UPDATE description ✨");
    println!("      └── update_package_license.sql     # UPDATE license ✨");
    println!();

    println!("✨ Что изменилось:");
    println!();
    println!("  ❌ БЫЛО (динамическое формирование SQL):");
    println!();
    println!("      let sql = format!(");
    println!("          \"UPDATE packages SET {{}} = ? WHERE name = ?\",");
    println!("          field_update.field_name()");
    println!("      );");
    println!();

    println!("  ✅ СТАЛО (SQL из файлов):");
    println!();
    println!("      let sql = field_update.sql_query();");
    println!();

    println!("🔧 Как это работает:");
    println!();
    println!("  1️⃣  Enum PackageFieldUpdate:");
    println!();
    println!("      pub enum PackageFieldUpdate {{");
    println!("          Name(String),");
    println!("          Version(String),");
    println!("          Repository(String),");
    println!("          Installed(bool),");
    println!("          Description(Option<String>),");
    println!("          License(Option<String>),");
    println!("      }}");
    println!();

    println!("  2️⃣  Метод sql_query() возвращает SQL из файла:");
    println!();
    println!("      impl PackageFieldUpdate {{");
    println!("          pub fn sql_query(&self) -> &'static str {{");
    println!("              match self {{");
    println!("                  Self::Name(_) => include_str!(\"../sql/queries/update_package_name.sql\"),");
    println!("                  Self::Version(_) => include_str!(\"../sql/queries/update_package_version.sql\"),");
    println!("                  // ...");
    println!("              }}");
    println!("          }}");
    println!("      }}");
    println!();

    println!("  3️⃣  Использование в database.rs:");
    println!();
    println!("      pub async fn update_package_field_in_database(");
    println!("          &self,");
    println!("          package: &Package,");
    println!("          field_update: PackageFieldUpdate,");
    println!("      ) -> Result<(), DataBaseError> {{");
    println!("          // ✅ SQL берётся из файла");
    println!("          let sql = field_update.sql_query();");
    println!("          ");
    println!("          let result = field_update");
    println!("              .bind_value(sqlx::query(sql))");
    println!("              .bind(&package.name)");
    println!("              .execute(&self.pool)");
    println!("              .await?;");
    println!("          ");
    println!("          if result.rows_affected() == 0 {{");
    println!("              return Err(DataBaseError::PackageNotFound(package.name.clone()));");
    println!("          }}");
    println!("          ");
    println!("          Ok(())");
    println!("      }}");
    println!();

    println!("📝 Содержимое SQL-файлов:");
    println!();
    println!("  update_package_name.sql:");
    println!("    UPDATE packages SET name = ? WHERE name = ?");
    println!();
    println!("  update_package_version.sql:");
    println!("    UPDATE packages SET version = ? WHERE name = ?");
    println!();
    println!("  update_package_installed.sql:");
    println!("    UPDATE packages SET installed = ? WHERE name = ?");
    println!();

    println!("✅ Преимущества:");
    println!();
    println!("  1. Все SQL в одном месте (папка sql/)");
    println!("     - Легко найти");
    println!("     - Легко редактировать");
    println!("     - Централизованное управление");
    println!();

    println!("  2. Проверка на этапе компиляции");
    println!("     - include_str!() встраивает файл");
    println!("     - Ошибка если файл не найден");
    println!("     - Zero runtime overhead");
    println!();

    println!("  3. Нет динамического формирования SQL");
    println!("     - Безопаснее");
    println!("     - Быстрее");
    println!("     - Проще отлаживать");
    println!();

    println!("  4. Версионирование в git");
    println!("     - История изменений SQL");
    println!("     - Code review запросов");
    println!();

    println!("📖 Пример использования:");
    println!();
    println!("  use PackageFieldUpdate;");
    println!();
    println!("  let package = db.get_package_from_database_by_name(\"nginx\").await?.unwrap();");
    println!();
    println!("  // Обновить версию");
    println!("  db.update_package_field_in_database(");
    println!("      &package,");
    println!("      PackageFieldUpdate::Version(\"1.25.0\".to_string())");
    println!("  ).await?;");
    println!();
    println!("  // SQL автоматически берётся из update_package_version.sql");
    println!();

    println!("🎯 Итог:");
    println!();
    println!("  Теперь ВСЕ SQL-запросы в проекте:");
    println!("  ✅ Хранятся в отдельных файлах");
    println!("  ✅ Загружаются через include_str!()");
    println!("  ✅ Проверяются на этапе компиляции");
    println!("  ✅ Нет динамического формирования SQL");
    println!();
    println!("  Единообразный подход во всём проекте!");
    println!();

    println!("=== Готово! ===");
}
