# Тест

Консольное приложение для проведения тестирования. Вопросы загружаются из YAML-файла, результаты оцениваются автоматически.

# Linux/macOS

```export TEST_PATH=test/philosophy/questions.yaml```  
```export TEST_COUNT=30```  
```export TEST_RESULT_DELAY=500ms```  
```go run ./cmd/app```

# Windows CMD

```set TEST_PATH=test/philosophy/questions.yaml```  
```set TEST_COUNT=30```  
```set TEST_RESULT_DELAY=500ms```  
```go run ./cmd/app```  