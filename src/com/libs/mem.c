#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

#define BUCKETS 32

struct HashElement {
    uint64_t loc;
    uint32_t val;
    struct HashElement *next;
    char initialized;
    char _padding[3];
};

uint32_t get_bucket(uint64_t loc) {
    return loc % BUCKETS;
}

struct HashElement *insert_val(struct HashElement *table_ptr[], uint64_t loc, uint32_t val) {
    uint32_t bucket = get_bucket(loc);

    struct HashElement *current = table_ptr[bucket];
    struct HashElement *next;

    if (current) {
        next = current->next;
    } else {
        current = (struct HashElement *) malloc(sizeof(struct HashElement));
        current->loc = loc;
        current->val = val;
        current->initialized = 0x1;
        
        current->next = (struct HashElement *) malloc(sizeof(struct HashElement));
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
    
    current->next = (struct HashElement *) malloc(sizeof(struct HashElement));
    current->next->initialized = 0x0;

    return next;
}

uint32_t get_val(struct HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = get_bucket(loc);

    struct HashElement *current = table_ptr[bucket];
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

uint32_t pop_element(struct HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = get_bucket(loc);

    struct HashElement *prev = 0x0;
    struct HashElement *current = table_ptr[bucket];
    while (current && current->loc != loc) {
        prev = current;
        current = current->next;
    }

    uint32_t val = current->val;
    struct HashElement *next = current->next;

    free(current);

    prev->next = next;

    return val;
}

int ensure_cells_right(struct HashElement *table_ptr[], uint64_t loc, size_t length) {
    uint32_t bucket = get_bucket(loc);

    for (int i = 0; i < length; i++) {
        if (get_val(table_ptr, loc + i) == 0) {
            insert_val(table_ptr, loc + i, 0);
        }
    }
}

int write_cells(struct HashElement *table_ptr[], uint64_t loc, size_t length) {
    // ensure_cells_right(table_ptr, loc, length);

    for (int i = 0; i < length; i++) {
        printf("%d ", get_val(table_ptr, loc + i));
    }
    
    printf("\n");
}

int init_table(struct HashElement *table_ptr[]) {
    for (int i = 0; i < BUCKETS; i++) {
        table_ptr[i] = 0x0;
    }
}

int free_table(struct HashElement *table_ptr[]) {
    for (int b = 0; b < BUCKETS; b++) {
        struct HashElement *current = table_ptr[b];
        // Deferencing null structures will give a pointer to 0x1
        // No idea where a pointer to 0xa00000000 came from...
        // especially after `init_table`
        while (current > 0x1 && current != 0xa00000000) { 
            struct HashElement *next = current->next;
            free(current);
            current = next;
        }
    }
}

