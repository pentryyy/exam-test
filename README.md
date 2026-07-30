# Тест

Консольное приложение для проведения тестирования. Вопросы загружаются из YAML-файла, результаты оцениваются автоматически.

# Linux/macOS

```export TEST_PATH=test/philosophy/questions.yaml```  
```export TEST_COUNT=30```  
```go run ./cmd/app```

# Windows CMD

```set TEST_PATH=test/philosophy/questions.yaml```  
```set TEST_COUNT=30```  
```go run ./cmd/app```  