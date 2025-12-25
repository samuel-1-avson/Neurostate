// Zephyr RTOS Code Generator
// Generates Zephyr API code for tasks, semaphores, mutexes, queues, timers

use super::*;

pub struct ZephyrHal;

impl ZephyrHal {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZephyrHal {
    fn default() -> Self {
        Self::new()
    }
}

impl RtosHal for ZephyrHal {
    fn rtos_type(&self) -> RtosType {
        RtosType::Zephyr
    }
    
    fn generate_task(&self, config: &TaskConfig) -> String {
        let priority = config.priority.to_zephyr();
        
        format!(r#"/**
 * Zephyr Thread: {name}
 * Stack: {stack} bytes, Priority: {priority}
 */

#include <zephyr/kernel.h>

#define {name_upper}_STACK_SIZE {stack}
#define {name_upper}_PRIORITY {priority}

K_THREAD_STACK_DEFINE({name}_stack, {name_upper}_STACK_SIZE);
static struct k_thread {name}_thread_data;
static k_tid_t {name}_tid;

void {entry}(void *p1, void *p2, void *p3) {{
    ARG_UNUSED(p1);
    ARG_UNUSED(p2);
    ARG_UNUSED(p3);
    
    // Thread initialization
    
    while (1) {{
        // Thread loop
        
        k_sleep(K_MSEC(100));
    }}
}}

void {name}_create(void) {{
    {name}_tid = k_thread_create(
        &{name}_thread_data,
        {name}_stack,
        {name_upper}_STACK_SIZE,
        {entry},
        NULL, NULL, NULL,
        {name_upper}_PRIORITY,
        0,
        K_NO_WAIT
    );
    k_thread_name_set({name}_tid, "{name}");
}}

void {name}_suspend(void) {{
    k_thread_suspend({name}_tid);
}}

void {name}_resume(void) {{
    k_thread_resume({name}_tid);
}}

void {name}_abort(void) {{
    k_thread_abort({name}_tid);
}}
"#,
            name = config.name,
            name_upper = config.name.to_uppercase(),
            entry = config.entry_function,
            stack = config.stack_size,
            priority = priority,
        )
    }
    
    fn generate_semaphore(&self, config: &SemaphoreConfig) -> String {
        match config.sem_type {
            SemaphoreType::Binary => format!(r#"/**
 * Zephyr Binary Semaphore: {name}
 */

#include <zephyr/kernel.h>

K_SEM_DEFINE({name}, {initial}, 1);

int {name}_take(k_timeout_t timeout) {{
    return k_sem_take(&{name}, timeout);
}}

void {name}_give(void) {{
    k_sem_give(&{name});
}}

unsigned int {name}_count(void) {{
    return k_sem_count_get(&{name});
}}

void {name}_reset(void) {{
    k_sem_reset(&{name});
}}
"#,
                name = config.name,
                initial = config.initial_count.min(1),
            ),
            SemaphoreType::Counting(max) => format!(r#"/**
 * Zephyr Counting Semaphore: {name}
 * Max: {max}, Initial: {initial}
 */

#include <zephyr/kernel.h>

K_SEM_DEFINE({name}, {initial}, {max});

int {name}_take(k_timeout_t timeout) {{
    return k_sem_take(&{name}, timeout);
}}

void {name}_give(void) {{
    k_sem_give(&{name});
}}

unsigned int {name}_count(void) {{
    return k_sem_count_get(&{name});
}}

void {name}_reset(void) {{
    k_sem_reset(&{name});
}}
"#,
                name = config.name,
                max = max,
                initial = config.initial_count,
            ),
        }
    }
    
    fn generate_mutex(&self, config: &MutexConfig) -> String {
        format!(r#"/**
 * Zephyr Mutex: {name}
 * Recursive: {recursive}
 */

#include <zephyr/kernel.h>

K_MUTEX_DEFINE({name});

int {name}_lock(k_timeout_t timeout) {{
    return k_mutex_lock(&{name}, timeout);
}}

int {name}_unlock(void) {{
    return k_mutex_unlock(&{name});
}}
"#,
            name = config.name,
            recursive = config.recursive,  // Zephyr mutexes are always recursive
        )
    }
    
    fn generate_queue(&self, config: &QueueConfig) -> String {
        format!(r#"/**
 * Zephyr Message Queue: {name}
 * Length: {length}, Item Size: {item_size} bytes
 */

#include <zephyr/kernel.h>

K_MSGQ_DEFINE({name}, {item_size}, {length}, 4);

int {name}_put(const void *data, k_timeout_t timeout) {{
    return k_msgq_put(&{name}, data, timeout);
}}

int {name}_get(void *data, k_timeout_t timeout) {{
    return k_msgq_get(&{name}, data, timeout);
}}

int {name}_peek(void *data) {{
    return k_msgq_peek(&{name}, data);
}}

void {name}_purge(void) {{
    k_msgq_purge(&{name});
}}

uint32_t {name}_num_used(void) {{
    return k_msgq_num_used_get(&{name});
}}

uint32_t {name}_num_free(void) {{
    return k_msgq_num_free_get(&{name});
}}
"#,
            name = config.name,
            length = config.length,
            item_size = config.item_size,
        )
    }
    
    fn generate_timer(&self, config: &TimerConfig) -> String {
        format!(r#"/**
 * Zephyr Timer: {name}
 * Period: {period}ms, Auto-reload: {auto_reload}
 */

#include <zephyr/kernel.h>

static void {callback}(struct k_timer *timer_id);

K_TIMER_DEFINE({name}, {callback}, NULL);

static void {callback}(struct k_timer *timer_id) {{
    ARG_UNUSED(timer_id);
    // Timer callback - runs every {period}ms
}}

void {name}_start(void) {{
    k_timer_start(&{name}, K_MSEC({period}), {duration});
}}

void {name}_stop(void) {{
    k_timer_stop(&{name});
}}

uint32_t {name}_status(void) {{
    return k_timer_status_get(&{name});
}}

uint32_t {name}_remaining(void) {{
    return k_timer_remaining_get(&{name});
}}
"#,
            name = config.name,
            period = config.period_ms,
            callback = config.callback,
            auto_reload = config.auto_reload,
            duration = if config.auto_reload { 
                format!("K_MSEC({})", config.period_ms) 
            } else { 
                "K_NO_WAIT".to_string() 
            },
        )
    }
    
    fn generate_event_group(&self, config: &EventGroupConfig) -> String {
        let bit_defs: String = (0..config.num_bits.min(32))
            .map(|i| format!("#define {}_BIT_{} BIT({})\n", config.name.to_uppercase(), i, i))
            .collect();
        
        format!(r#"/**
 * Zephyr Event: {name}
 * Bits: {num_bits}
 */

#include <zephyr/kernel.h>

{bit_defs}
K_EVENT_DEFINE({name});

uint32_t {name}_set(uint32_t events) {{
    return k_event_set(&{name}, events);
}}

uint32_t {name}_clear(uint32_t events) {{
    return k_event_clear(&{name}, events);
}}

uint32_t {name}_wait(uint32_t events, bool wait_all, k_timeout_t timeout) {{
    uint32_t options = wait_all ? K_EVENT_WAIT_ALL : K_EVENT_WAIT_ANY;
    return k_event_wait(&{name}, events, options, timeout);
}}

uint32_t {name}_post(uint32_t events) {{
    return k_event_post(&{name}, events);
}}
"#,
            name = config.name,
            num_bits = config.num_bits,
            bit_defs = bit_defs,
        )
    }
    
    fn generate_config_header(&self) -> String {
        r#"/**
 * Zephyr prj.conf
 * Auto-generated by NeuroBench
 */

# General
CONFIG_MAIN_STACK_SIZE=2048
CONFIG_HEAP_MEM_POOL_SIZE=16384

# Kernel Options
CONFIG_MULTITHREADING=y
CONFIG_NUM_PREEMPT_PRIORITIES=16
CONFIG_NUM_COOP_PRIORITIES=16
CONFIG_TIMESLICING=y
CONFIG_TIMESLICE_SIZE=10

# Semaphores
CONFIG_SEMAPHORE=y

# Mutexes
CONFIG_MUTEX=y

# Message Queues
CONFIG_MSGQ=y

# Timers
CONFIG_TIMER=y

# Events
CONFIG_EVENTS=y

# Memory Management
CONFIG_MEM_SLAB=y
CONFIG_MEMPOOL=y

# Logging
CONFIG_LOG=y
CONFIG_LOG_DEFAULT_LEVEL=3
CONFIG_LOG_BACKEND_UART=y

# Debug
CONFIG_DEBUG=y
CONFIG_ASSERT=y
CONFIG_THREAD_MONITOR=y
CONFIG_THREAD_NAME=y
CONFIG_THREAD_STACK_INFO=y

# Power Management
CONFIG_PM=y
CONFIG_PM_DEVICE=y
"#.to_string()
    }
    
    fn generate_main(&self, tasks: &[TaskConfig]) -> String {
        let thread_creates: String = tasks.iter()
            .filter(|t| t.auto_start)
            .map(|t| format!("    {}_create();\n", t.name))
            .collect();
        
        format!(r#"/**
 * Zephyr Main Application
 * Auto-generated by NeuroBench
 */

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>

LOG_MODULE_REGISTER(app, LOG_LEVEL_INF);

// Thread declarations
{thread_externs}

int main(void) {{
    LOG_INF("NeuroBench Zephyr Application Starting...");
    
    // Create threads
{thread_creates}
    
    // Main thread can do work or sleep
    while (1) {{
        k_sleep(K_SECONDS(1));
    }}
    
    return 0;
}}
"#,
            thread_externs = tasks.iter()
                .map(|t| format!("extern void {}_create(void);\n", t.name))
                .collect::<String>(),
            thread_creates = thread_creates,
        )
    }
}
