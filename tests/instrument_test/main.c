#include <stdio.h>
#include "math_utils.h"

int main() {
    // ABSTRAKTOR_BLOCK_EVENT
    int a = 5;
    int b = 3;
    
    int result = add(a, b);
    
    printf("Result: %d\n", result);

    {
        // ABSTRAKTOR_BLOCK_EVENT
        int square_result = square(result);
        printf("Square: %d\n", square_result);
    }

    return 0;
} 