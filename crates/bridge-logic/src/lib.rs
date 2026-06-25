//! Крейт `bridge-logic` — ядро трансформации событий моста Gitea → Jenkins.
//!
//! Содержит `EventProcessor`, преобразующий вебхуки Gitea (push, pull request)
//! в запросы на параметризованный запуск сборок Jenkins без сетевых обращений.

/// Модуль с `EventProcessor` и моделями запроса на запуск сборки Jenkins.
pub mod processor;
