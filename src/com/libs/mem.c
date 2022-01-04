#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

#define BUCKETS 32

typedef struct HashElement {
    uint64_t loc;
    uint32_t val;
    struct HashElement *next;
    char initialized;
    char _padding[3];
} HashElement;

uint32_t get_bucket(uint64_t loc) {
    return loc % BUCKETS;
}

struct HashElement *insert_val(HashElement *table_ptr[], uint64_t loc, uint32_t val) {
    uint32_t bucket = get_bucket(loc);

    HashElement *current = table_ptr[bucket];
    HashElement *next;

    if (current) {
        next = current->next;
    } else {
        current = (HashElement *) malloc(sizeof(HashElement));
        current->loc = loc;
        current->val = val;
        current->initialized = 0x1;
        
        current->next = (HashElement *) malloc(sizeof(HashElement));
        current->next->initialized = 0x0;
        
        table_ptr[bucket] = current;

        return current;
    }

    size_t i = 0;
    while (next) {
        current = next;
        next = current->next;

        i++;
    }
    
    current->loc = loc;
    current->val = val;
    current->initialized = 0x1;
    
    current->next = (HashElement *) malloc(sizeof(HashElement));
    current->next->initialized = 0x0;

    return next;
}

uint32_t get_val(HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = get_bucket(loc);

    HashElement *current = table_ptr[bucket];
    while (current && current->initialized == 0x1) {
        if (current->loc == loc) {
            return current->val;
        } else {
            current = current->next;
        }
    }
    
    uint32_t val = 0;
    if (current && current->val) {
        val = current->val;
    }

    return val;
}

uint32_t pop_element(HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = get_bucket(loc);

    HashElement *prev = 0x0;
    HashElement *current = table_ptr[bucket];
    while (current && current->loc != loc) {
        prev = current;
        current = current->next;
    }

    uint32_t val = current->val;
    HashElement *next = current->next;

    if (!prev) {
        table_ptr[bucket] = next;
    } else {
        prev->next = next;
        free(current);
    }
    
    return val;
}

int ensure_cells_right(HashElement *table_ptr[], uint64_t loc, size_t length) {
    uint32_t bucket = get_bucket(loc);

    for (int i = 0; i < length; i++) {
        if (get_val(table_ptr, loc + i) == 0) {
            insert_val(table_ptr, loc + i, 0);
        }
    }
}

int write_cells(HashElement *table_ptr[], uint64_t loc, size_t length) {
    // ensure_cells_right(table_ptr, loc, length);

    for (int i = 0; i < length; i++) {
        printf("%d ", get_val(table_ptr, loc + i));
    }
    
    printf("\n");
}

int init_table(HashElement *table_ptr[]) {
    for (int i = 0; i < BUCKETS; i++) {
        table_ptr[i] = 0x0;
    }
}

int free_table(HashElement *table_ptr[]) {
    for (int b = 0; b < BUCKETS; b++) {
        HashElement *current = table_ptr[b];
        // Deferencing null structures will give a pointer to 0x1
        // No idea where a pointer to 0xa00000000 came from...
        // especially after `init_table`
        while (current > 0x1 && current != 0xa00000000) { 
            HashElement *next = current->next;
            free(current);
            current = next;
        }
    }
}

