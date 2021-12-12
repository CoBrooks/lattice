#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

#define BUCKETS 32

struct HashElement {
    uint64_t loc;
    uint32_t val;
    struct HashElement *next;
};

struct HashElement *insert_val(struct HashElement *table_ptr[], uint64_t loc, uint32_t val) {
    uint32_t bucket = loc / (4294967295 / BUCKETS);

    size_t i = 0;
    struct HashElement *prev = 0x0;
    struct HashElement *current = table_ptr[bucket];

    while (current) {
        prev = current;
        current = current->next;
        i++;
    }

    struct HashElement *new;
    new = (struct HashElement *) malloc(sizeof(struct HashElement)); 
    new->loc = loc;
    new->val = val;
    new->next = NULL;

    if (i == 0) {
        table_ptr[bucket] = new;
    }

    if (prev) {
        prev->next = new;
    }

    return new; 
}

uint32_t get_val(struct HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = loc / (4294967295 / BUCKETS);
    
    struct HashElement *current = table_ptr[bucket];
    while (current && current->loc != loc) {
        current = current->next;
    }

    uint32_t val = 0;
    if (current && current->val) {
        val =  current->val;
    }

    return val;
}

uint32_t pop_element(struct HashElement *table_ptr[], uint64_t loc) {
    uint32_t bucket = loc / (4294967295 / BUCKETS);

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
    uint32_t bucket = loc / (4294967295 / BUCKETS);

    for (int i = 0; i < length; i++) {
        if (get_val(table_ptr, loc + i) == 0) {
            insert_val(table_ptr, loc + i, 0);
        }
    }
}

int write_cells(struct HashElement *table_ptr[], uint64_t loc, size_t length) {
    ensure_cells_right(table_ptr, loc, length);

    for (int i = 0; i < length; i++) {
        printf("%d ", get_val(table_ptr, loc + i));
    }
    
    printf("\n");
}

int init_table(struct HashElement *table_ptr[]) {
    for (int b = 0; b < BUCKETS; b++) {
        table_ptr[b] = 0x0;
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

