#include <stdlib.h>
#include <stdint.h>

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
    
    return current->val;
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

int init_table(struct HashElement *table_ptr[]) {
    for (int b = 0; b < BUCKETS; b++) {
        table_ptr[b] = 0x0;
    }
}

int free_table(struct HashElement *table_ptr[]) {
    for (int b = 0; b < BUCKETS; b++) {
        struct HashElement *current = table_ptr[b];
        // Deferencing null structures will give a pointer to 0x1
        while (current > 0x1) {             
            struct HashElement *next = current->next;
            free(current);
            current = next;
        }
    }
}

