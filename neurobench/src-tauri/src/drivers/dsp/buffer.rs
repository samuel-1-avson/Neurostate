// Circular Buffer Generator
// Thread-safe and non-thread-safe circular buffer implementations

use super::*;

/// Generate circular buffer C code
pub fn generate_buffer_code(config: &CircularBufferConfig) -> String {
    let thread_safe_code = if config.thread_safe {
        format!(r#"
// Thread-safe operations using atomic compare-and-swap
#include "cmsis_os.h"

static osMutexId_t {name}_mutex;
static const osMutexAttr_t {name}_mutex_attr = {{
    .name = "{name}_mutex",
    .attr_bits = osMutexRecursive,
}};

void {name}_lock(void) {{
    osMutexAcquire({name}_mutex, osWaitForever);
}}

void {name}_unlock(void) {{
    osMutexRelease({name}_mutex);
}}

void {name}_init_mutex(void) {{
    {name}_mutex = osMutexNew(&{name}_mutex_attr);
}}
"#,
            name = config.name,
        )
    } else {
        String::new()
    };

    let lock_call = if config.thread_safe { format!("{}_lock();", config.name) } else { String::new() };
    let unlock_call = if config.thread_safe { format!("{}_unlock();", config.name) } else { String::new() };

    format!(r#"/**
 * Circular Buffer: {name}
 * Size: {size} elements
 * Element Type: {elem_type}
 * Thread-safe: {thread_safe}
 */

#include <stdint.h>
#include <stdbool.h>
#include <string.h>

#define {name_upper}_SIZE {size}

typedef struct {{
    {elem_type} buffer[{name_upper}_SIZE];
    volatile uint32_t head;
    volatile uint32_t tail;
    volatile uint32_t count;
}} circular_buffer_t;

static circular_buffer_t {name};
{thread_safe_code}
void {name}_init(void) {{
    memset({name}.buffer, 0, sizeof({name}.buffer));
    {name}.head = 0;
    {name}.tail = 0;
    {name}.count = 0;
{mutex_init}}}

bool {name}_is_empty(void) {{
    return {name}.count == 0;
}}

bool {name}_is_full(void) {{
    return {name}.count == {name_upper}_SIZE;
}}

uint32_t {name}_available(void) {{
    return {name}.count;
}}

uint32_t {name}_free_space(void) {{
    return {name_upper}_SIZE - {name}.count;
}}

bool {name}_push({elem_type} value) {{
    {lock}
    if ({name}_is_full()) {{
        {unlock}
        return false;
    }}
    
    {name}.buffer[{name}.head] = value;
    {name}.head = ({name}.head + 1) % {name_upper}_SIZE;
    {name}.count++;
    {unlock}
    return true;
}}

bool {name}_pop({elem_type} *value) {{
    {lock}
    if ({name}_is_empty()) {{
        {unlock}
        return false;
    }}
    
    *value = {name}.buffer[{name}.tail];
    {name}.tail = ({name}.tail + 1) % {name_upper}_SIZE;
    {name}.count--;
    {unlock}
    return true;
}}

bool {name}_peek({elem_type} *value) {{
    if ({name}_is_empty()) {{
        return false;
    }}
    *value = {name}.buffer[{name}.tail];
    return true;
}}

// Push multiple elements
uint32_t {name}_push_bulk(const {elem_type} *data, uint32_t len) {{
    {lock}
    uint32_t pushed = 0;
    
    for (uint32_t i = 0; i < len && !{name}_is_full(); i++) {{
        {name}.buffer[{name}.head] = data[i];
        {name}.head = ({name}.head + 1) % {name_upper}_SIZE;
        {name}.count++;
        pushed++;
    }}
    {unlock}
    return pushed;
}}

// Pop multiple elements  
uint32_t {name}_pop_bulk({elem_type} *data, uint32_t len) {{
    {lock}
    uint32_t popped = 0;
    
    for (uint32_t i = 0; i < len && !{name}_is_empty(); i++) {{
        data[i] = {name}.buffer[{name}.tail];
        {name}.tail = ({name}.tail + 1) % {name_upper}_SIZE;
        {name}.count--;
        popped++;
    }}
    {unlock}
    return popped;
}}

// Overwrite oldest data if full (useful for streaming)
void {name}_push_overwrite({elem_type} value) {{
    {lock}
    if ({name}_is_full()) {{
        // Move tail forward, discarding oldest
        {name}.tail = ({name}.tail + 1) % {name_upper}_SIZE;
        {name}.count--;
    }}
    
    {name}.buffer[{name}.head] = value;
    {name}.head = ({name}.head + 1) % {name_upper}_SIZE;
    {name}.count++;
    {unlock}
}}

void {name}_clear(void) {{
    {lock}
    {name}.head = 0;
    {name}.tail = 0;
    {name}.count = 0;
    {unlock}
}}

// Get pointer to contiguous read region (for DMA)
{elem_type} *{name}_get_read_ptr(uint32_t *available) {{
    uint32_t tail = {name}.tail;
    uint32_t head = {name}.head;
    
    if (head >= tail) {{
        *available = head - tail;
    }} else {{
        *available = {name_upper}_SIZE - tail;  // Up to end of buffer
    }}
    
    return &{name}.buffer[tail];
}}

// Advance read pointer after DMA read
void {name}_advance_read(uint32_t count) {{
    {lock}
    {name}.tail = ({name}.tail + count) % {name_upper}_SIZE;
    {name}.count -= count;
    {unlock}
}}
"#,
        name = config.name,
        name_upper = config.name.to_uppercase(),
        size = config.size,
        elem_type = config.element_type,
        thread_safe = config.thread_safe,
        thread_safe_code = thread_safe_code,
        lock = lock_call,
        unlock = unlock_call,
        mutex_init = if config.thread_safe { 
            format!("    {}_init_mutex();", config.name) 
        } else { 
            String::new() 
        },
    )
}
