# Learning Progress — AI Engineering

**Дата начала:** 2026-02-14
**Последнее обновление:** 2026-02-15
**Obsidian vault:** `~/ObsidanVaults/ML/` (Progress.md — основной трекер)

## Общий прогресс

| Модуль | Статус | Прогресс |
|--------|--------|----------|
| 1. LLM изнутри | В процессе | 60% |
| 2. Embeddings глубоко | Не начат | 0% |
| 3. Evaluation | Не начат | 0% |
| 4. Prompt Engineering & Advanced RAG | Не начат | 0% |
| 5. AI Agents | Не начат | 0% |
| 6. Fine-tuning | Не начат | 0% |

## Уровень по областям

| Область | Уровень | Заметки |
|---------|---------|---------|
| LLM internals | Начальный+ | Понимает tokenization (BPE/WordPiece), trade-offs словаря, softmax |
| Embeddings | Начальный | Использует API, метрики не выбирал осознанно |
| Evaluation | Начальный | Проверял вручную, без метрик |
| Prompt Engineering | Базовый | Пишет промпты, но не систематически |
| AI Agents | Начальный | Пользовался Claude Code/ChatGPT, не строил |
| Fine-tuning | Теория | Читал, не практиковал |
| Python/ML | Базовый | scikit-learn, numpy |

## Детальный прогресс по модулям

### Модуль 1: LLM изнутри
- [x] Tokenization (BPE, WordPiece)
- [ ] Transformer архитектура
- [ ] Attention mechanism (Q/K/V)
- [x] Параметры генерации
- [x] Контекстное окно, KV-cache
- [ ] **Проект:** CLI-утилита сравнения моделей

### Модуль 2: Embeddings глубоко
- [ ] Архитектуры embedding-моделей
- [ ] Метрики расстояния
- [ ] Chunking стратегии
- [ ] Индексы (HNSW, IVF, PQ)
- [ ] **Проект:** A/B сравнение стратегий в RAG

### Модуль 3: Evaluation
- [ ] Метрики RAG
- [ ] RAGAS framework
- [ ] LLM-as-judge
- [ ] Golden datasets
- [ ] **Проект:** Evaluation pipeline

### Модуль 4: Prompt Engineering & Advanced RAG
- [ ] Систематический prompt engineering
- [ ] Hybrid search
- [ ] Reranking
- [ ] Query decomposition, HyDE
- [ ] **Проект:** Advanced RAG с измерением улучшений

### Модуль 5: AI Agents
- [ ] Tool use / function calling
- [ ] ReAct pattern
- [ ] Memory (short-term, long-term)
- [ ] MCP
- [ ] Multi-agent архитектуры
- [ ] **Проект:** AI-агент "второй мозг"

### Модуль 6: Fine-tuning
- [ ] Decision framework (когда что использовать)
- [ ] Подготовка данных
- [ ] LoRA/QLoRA
- [ ] Training & evaluation
- [ ] **Проект:** Fine-tune модели для агента

## Журнал сессий

| Дата | Модуль | Тема | Что сделано |
|------|--------|------|-------------|
| 2026-02-14 | — | Диагностика | Определён начальный уровень, создан план обучения |
| 2026-02-15 | M1 | Tokenization | BPE/WordPiece/SentencePiece, trade-offs словаря, мини-тест |
| 2026-02-15 | M1 | Transformer | Создан справочный документ по полной архитектуре |
| 2026-02-15 | — | Реструктуризация | Obsidian vault переделан под 6 модулей |
| 2026-02-15 | M1 | Параметры генерации | top-k, temperature (contrast-эффект), frequency/presence penalty. Мини-тест: 3/3 (1 частично) |
| 2026-02-15 | M1 | KV-cache | Контекстное окно, KV-cache механизм, стоимость, оптимизации (MQA/GQA), prompt caching. Мини-тест: 2/2 (1 частично) |
