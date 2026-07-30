package cli

import (
	"fmt"
	"os"
)

func LogError(format string, args ...any) bool {
	_, err := fmt.Fprintf(os.Stderr, format+"\n", args...)
	return err == nil
}
