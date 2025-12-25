// FreeRTOS Code Generator
// Generates FreeRTOS API code for tasks, semaphores, mutexes, queues, timers

use super::*;

pub struct FreeRtosHal;

impl FreeRtosHal {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FreeRtosHal {
    fn default() -> Self {
        Self::new()
    }
}

impl RtosHal for FreeRtosHal {
    fn rtos_type(&self) -> RtosType {
        RtosType::FreeRtos
    }
    
    fn generate_task(&self, config: &TaskConfig) -> String {
        let priority = config.priority.to_freertos();
        let param = config.parameter.as_deref().unwrap_or("NULL");
        
        format!(r#"/**
 * FreeRTOS Task: {name}
 * Stack: {stack} words, Priority: {priority}
 */

#include "FreeRTOS.h"
#include "task.h"

static TaskHandle_t x{name}Handle = NULL;

void {entry}(void *pvParameters) {{
    // Task initialization
    
    for (;;) {{
        // Task loop
        
        vTaskDelay(pdMS_TO_TICKS(100));
    }}
}}

void {name}_Create(void) {{
    BaseType_t xReturned = xTaskCreate(
        {entry},           // Task function
        "{name}",          // Task name
        {stack},           // Stack size (words)
        {param},           // Parameters
        {priority},        // Priority
        &x{name}Handle     // Task handle
    );
    
    configASSERT(xReturned == pdPASS);
}}

void {name}_Delete(void) {{
    if (x{name}Handle != NULL) {{
        vTaskDelete(x{name}Handle);
        x{name}Handle = NULL;
    }}
}}

void {name}_Suspend(void) {{
    if (x{name}Handle != NULL) {{
        vTaskSuspend(x{name}Handle);
    }}
}}

void {name}_Resume(void) {{
    if (x{name}Handle != NULL) {{
        vTaskResume(x{name}Handle);
    }}
}}
"#,
            name = config.name,
            entry = config.entry_function,
            stack = config.stack_size / 4,  // Convert bytes to words
            priority = priority,
            param = param,
        )
    }
    
    fn generate_semaphore(&self, config: &SemaphoreConfig) -> String {
        match config.sem_type {
            SemaphoreType::Binary => format!(r#"/**
 * FreeRTOS Binary Semaphore: {name}
 */

#include "FreeRTOS.h"
#include "semphr.h"

static SemaphoreHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xSemaphoreCreateBinary();
    configASSERT(x{name} != NULL);
{give}}}

BaseType_t {name}_Take(TickType_t xTicksToWait) {{
    return xSemaphoreTake(x{name}, xTicksToWait);
}}

BaseType_t {name}_Give(void) {{
    return xSemaphoreGive(x{name});
}}

BaseType_t {name}_GiveFromISR(BaseType_t *pxHigherPriorityTaskWoken) {{
    return xSemaphoreGiveFromISR(x{name}, pxHigherPriorityTaskWoken);
}}
"#,
                name = config.name,
                give = if config.initial_count > 0 { format!("    xSemaphoreGive(x{});\n", config.name) } else { String::new() },
            ),
            SemaphoreType::Counting(max) => format!(r#"/**
 * FreeRTOS Counting Semaphore: {name}
 * Max: {max}, Initial: {initial}
 */

#include "FreeRTOS.h"
#include "semphr.h"

static SemaphoreHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xSemaphoreCreateCounting({max}, {initial});
    configASSERT(x{name} != NULL);
}}

BaseType_t {name}_Take(TickType_t xTicksToWait) {{
    return xSemaphoreTake(x{name}, xTicksToWait);
}}

BaseType_t {name}_Give(void) {{
    return xSemaphoreGive(x{name});
}}

UBaseType_t {name}_GetCount(void) {{
    return uxSemaphoreGetCount(x{name});
}}
"#,
                name = config.name,
                max = max,
                initial = config.initial_count,
            ),
        }
    }
    
    fn generate_mutex(&self, config: &MutexConfig) -> String {
        if config.recursive {
            format!(r#"/**
 * FreeRTOS Recursive Mutex: {name}
 */

#include "FreeRTOS.h"
#include "semphr.h"

static SemaphoreHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xSemaphoreCreateRecursiveMutex();
    configASSERT(x{name} != NULL);
}}

BaseType_t {name}_Lock(TickType_t xTicksToWait) {{
    return xSemaphoreTakeRecursive(x{name}, xTicksToWait);
}}

BaseType_t {name}_Unlock(void) {{
    return xSemaphoreGiveRecursive(x{name});
}}
"#,
                name = config.name,
            )
        } else {
            format!(r#"/**
 * FreeRTOS Mutex: {name}
 */

#include "FreeRTOS.h"
#include "semphr.h"

static SemaphoreHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xSemaphoreCreateMutex();
    configASSERT(x{name} != NULL);
}}

BaseType_t {name}_Lock(TickType_t xTicksToWait) {{
    return xSemaphoreTake(x{name}, xTicksToWait);
}}

BaseType_t {name}_Unlock(void) {{
    return xSemaphoreGive(x{name});
}}

TaskHandle_t {name}_GetHolder(void) {{
    return xSemaphoreGetMutexHolder(x{name});
}}
"#,
                name = config.name,
            )
        }
    }
    
    fn generate_queue(&self, config: &QueueConfig) -> String {
        format!(r#"/**
 * FreeRTOS Queue: {name}
 * Length: {length}, Item Size: {item_size} bytes
 */

#include "FreeRTOS.h"
#include "queue.h"

static QueueHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xQueueCreate({length}, {item_size});
    configASSERT(x{name} != NULL);
}}

BaseType_t {name}_Send(const void *pvItemToQueue, TickType_t xTicksToWait) {{
    return xQueueSend(x{name}, pvItemToQueue, xTicksToWait);
}}

BaseType_t {name}_SendToFront(const void *pvItemToQueue, TickType_t xTicksToWait) {{
    return xQueueSendToFront(x{name}, pvItemToQueue, xTicksToWait);
}}

BaseType_t {name}_Receive(void *pvBuffer, TickType_t xTicksToWait) {{
    return xQueueReceive(x{name}, pvBuffer, xTicksToWait);
}}

BaseType_t {name}_Peek(void *pvBuffer, TickType_t xTicksToWait) {{
    return xQueuePeek(x{name}, pvBuffer, xTicksToWait);
}}

UBaseType_t {name}_MessagesWaiting(void) {{
    return uxQueueMessagesWaiting(x{name});
}}

BaseType_t {name}_SendFromISR(const void *pvItemToQueue, BaseType_t *pxHigherPriorityTaskWoken) {{
    return xQueueSendFromISR(x{name}, pvItemToQueue, pxHigherPriorityTaskWoken);
}}

BaseType_t {name}_ReceiveFromISR(void *pvBuffer, BaseType_t *pxHigherPriorityTaskWoken) {{
    return xQueueReceiveFromISR(x{name}, pvBuffer, pxHigherPriorityTaskWoken);
}}
"#,
            name = config.name,
            length = config.length,
            item_size = config.item_size,
        )
    }
    
    fn generate_timer(&self, config: &TimerConfig) -> String {
        format!(r#"/**
 * FreeRTOS Software Timer: {name}
 * Period: {period}ms, Auto-reload: {auto_reload}
 */

#include "FreeRTOS.h"
#include "timers.h"

static TimerHandle_t x{name} = NULL;

static void {callback}(TimerHandle_t xTimer) {{
    // Timer callback - runs every {period}ms
    (void)xTimer;
}}

void {name}_Create(void) {{
    x{name} = xTimerCreate(
        "{name}",                      // Timer name
        pdMS_TO_TICKS({period}),       // Period
        {reload},                      // Auto-reload
        (void *)0,                     // Timer ID
        {callback}                     // Callback
    );
    configASSERT(x{name} != NULL);
}}

BaseType_t {name}_Start(TickType_t xTicksToWait) {{
    return xTimerStart(x{name}, xTicksToWait);
}}

BaseType_t {name}_Stop(TickType_t xTicksToWait) {{
    return xTimerStop(x{name}, xTicksToWait);
}}

BaseType_t {name}_Reset(TickType_t xTicksToWait) {{
    return xTimerReset(x{name}, xTicksToWait);
}}

BaseType_t {name}_ChangePeriod(TickType_t xNewPeriod, TickType_t xTicksToWait) {{
    return xTimerChangePeriod(x{name}, xNewPeriod, xTicksToWait);
}}

BaseType_t {name}_IsActive(void) {{
    return xTimerIsTimerActive(x{name});
}}
"#,
            name = config.name,
            period = config.period_ms,
            callback = config.callback,
            auto_reload = config.auto_reload,
            reload = if config.auto_reload { "pdTRUE" } else { "pdFALSE" },
        )
    }
    
    fn generate_event_group(&self, config: &EventGroupConfig) -> String {
        let bit_defs: String = (0..config.num_bits.min(24))
            .map(|i| format!("#define {}_BIT_{} (1 << {})\n", config.name.to_uppercase(), i, i))
            .collect();
        
        format!(r#"/**
 * FreeRTOS Event Group: {name}
 * Bits: {num_bits}
 */

#include "FreeRTOS.h"
#include "event_groups.h"

{bit_defs}
static EventGroupHandle_t x{name} = NULL;

void {name}_Create(void) {{
    x{name} = xEventGroupCreate();
    configASSERT(x{name} != NULL);
}}

EventBits_t {name}_SetBits(EventBits_t uxBitsToSet) {{
    return xEventGroupSetBits(x{name}, uxBitsToSet);
}}

EventBits_t {name}_ClearBits(EventBits_t uxBitsToClear) {{
    return xEventGroupClearBits(x{name}, uxBitsToClear);
}}

EventBits_t {name}_GetBits(void) {{
    return xEventGroupGetBits(x{name});
}}

EventBits_t {name}_WaitBits(
    EventBits_t uxBitsToWaitFor,
    BaseType_t xClearOnExit,
    BaseType_t xWaitForAllBits,
    TickType_t xTicksToWait
) {{
    return xEventGroupWaitBits(x{name}, uxBitsToWaitFor, xClearOnExit, xWaitForAllBits, xTicksToWait);
}}

EventBits_t {name}_Sync(
    EventBits_t uxBitsToSet,
    EventBits_t uxBitsToWaitFor,
    TickType_t xTicksToWait
) {{
    return xEventGroupSync(x{name}, uxBitsToSet, uxBitsToWaitFor, xTicksToWait);
}}
"#,
            name = config.name,
            num_bits = config.num_bits,
            bit_defs = bit_defs,
        )
    }
    
    fn generate_config_header(&self) -> String {
        r#"/**
 * FreeRTOSConfig.h
 * Auto-generated by NeuroBench
 */

#ifndef FREERTOS_CONFIG_H
#define FREERTOS_CONFIG_H

/* Kernel configuration */
#define configUSE_PREEMPTION                    1
#define configUSE_PORT_OPTIMISED_TASK_SELECTION 1
#define configUSE_TICKLESS_IDLE                 0
#define configCPU_CLOCK_HZ                      (SystemCoreClock)
#define configTICK_RATE_HZ                      ((TickType_t)1000)
#define configMAX_PRIORITIES                    (7)
#define configMINIMAL_STACK_SIZE                ((uint16_t)128)
#define configMAX_TASK_NAME_LEN                 (16)
#define configUSE_16_BIT_TICKS                  0
#define configIDLE_SHOULD_YIELD                 1
#define configUSE_TASK_NOTIFICATIONS            1
#define configTASK_NOTIFICATION_ARRAY_ENTRIES   3

/* Memory allocation */
#define configSUPPORT_STATIC_ALLOCATION         1
#define configSUPPORT_DYNAMIC_ALLOCATION        1
#define configTOTAL_HEAP_SIZE                   ((size_t)(32 * 1024))
#define configAPPLICATION_ALLOCATED_HEAP        0

/* Hook function related */
#define configUSE_IDLE_HOOK                     0
#define configUSE_TICK_HOOK                     0
#define configCHECK_FOR_STACK_OVERFLOW          2
#define configUSE_MALLOC_FAILED_HOOK            1
#define configUSE_DAEMON_TASK_STARTUP_HOOK      0

/* Run time and task stats */
#define configGENERATE_RUN_TIME_STATS           0
#define configUSE_TRACE_FACILITY                1
#define configUSE_STATS_FORMATTING_FUNCTIONS    1

/* Co-routine related */
#define configUSE_CO_ROUTINES                   0
#define configMAX_CO_ROUTINE_PRIORITIES         (2)

/* Software timer */
#define configUSE_TIMERS                        1
#define configTIMER_TASK_PRIORITY               (configMAX_PRIORITIES - 1)
#define configTIMER_QUEUE_LENGTH                10
#define configTIMER_TASK_STACK_DEPTH            (configMINIMAL_STACK_SIZE * 2)

/* Mutexes */
#define configUSE_MUTEXES                       1
#define configUSE_RECURSIVE_MUTEXES             1

/* Semaphores */
#define configUSE_COUNTING_SEMAPHORES           1

/* Queues */
#define configQUEUE_REGISTRY_SIZE               8

/* Event Groups */
#define configUSE_EVENT_GROUPS                  1

/* Interrupt nesting */
#define configKERNEL_INTERRUPT_PRIORITY         (255)
#define configMAX_SYSCALL_INTERRUPT_PRIORITY    (191)
#define configLIBRARY_KERNEL_INTERRUPT_PRIORITY 15
#define configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY 5

/* Assert */
#define configASSERT(x) if((x) == 0) { taskDISABLE_INTERRUPTS(); for(;;); }

/* FreeRTOS MPU specific */
#define configINCLUDE_APPLICATION_DEFINED_PRIVILEGED_FUNCTIONS 0
#define configTOTAL_MPU_REGIONS                 8

/* Optional functions */
#define INCLUDE_vTaskPrioritySet                1
#define INCLUDE_uxTaskPriorityGet               1
#define INCLUDE_vTaskDelete                     1
#define INCLUDE_vTaskSuspend                    1
#define INCLUDE_xResumeFromISR                  1
#define INCLUDE_vTaskDelayUntil                 1
#define INCLUDE_vTaskDelay                      1
#define INCLUDE_xTaskGetSchedulerState          1
#define INCLUDE_xTaskGetCurrentTaskHandle       1
#define INCLUDE_uxTaskGetStackHighWaterMark     1
#define INCLUDE_xTaskGetIdleTaskHandle          1
#define INCLUDE_eTaskGetState                   1
#define INCLUDE_xEventGroupSetBitFromISR        1
#define INCLUDE_xTimerPendFunctionCall          1
#define INCLUDE_xTaskAbortDelay                 1
#define INCLUDE_xTaskGetHandle                  1
#define INCLUDE_xTaskResumeFromISR              1

/* Newlib reentrancy */
#define configUSE_NEWLIB_REENTRANT              0

/* Cortex-M specific */
#ifdef __NVIC_PRIO_BITS
    #define configPRIO_BITS __NVIC_PRIO_BITS
#else
    #define configPRIO_BITS 4
#endif

/* Handlers */
#define vPortSVCHandler    SVC_Handler
#define xPortPendSVHandler PendSV_Handler
#define xPortSysTickHandler SysTick_Handler

#endif /* FREERTOS_CONFIG_H */
"#.to_string()
    }
    
    fn generate_main(&self, tasks: &[TaskConfig]) -> String {
        let task_creates: String = tasks.iter()
            .filter(|t| t.auto_start)
            .map(|t| format!("    {}_Create();\n", t.name))
            .collect();
        
        format!(r#"/**
 * FreeRTOS Main Application
 * Auto-generated by NeuroBench
 */

#include "FreeRTOS.h"
#include "task.h"

// Task declarations
{task_externs}

int main(void) {{
    // Hardware initialization
    SystemClock_Config();
    HAL_Init();
    
    // Create RTOS objects
{task_creates}
    
    // Start scheduler
    vTaskStartScheduler();
    
    // Should never reach here
    for (;;) {{}}
    
    return 0;
}}

void vApplicationStackOverflowHook(TaskHandle_t xTask, char *pcTaskName) {{
    (void)xTask;
    (void)pcTaskName;
    // Stack overflow detected
    taskDISABLE_INTERRUPTS();
    for (;;) {{}}
}}

void vApplicationMallocFailedHook(void) {{
    // Malloc failed
    taskDISABLE_INTERRUPTS();
    for (;;) {{}}
}}

void vApplicationIdleHook(void) {{
    // Idle hook - can enter low power mode
}}

void vApplicationTickHook(void) {{
    // Tick hook
}}
"#,
            task_externs = tasks.iter()
                .map(|t| format!("extern void {}(void *pvParameters);\n", t.entry_function))
                .collect::<String>(),
            task_creates = task_creates,
        )
    }
}
