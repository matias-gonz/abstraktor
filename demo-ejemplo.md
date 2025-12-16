Demo Abstraktor - Ejemplo Worker
================================

CÓDIGO CON BUG
--------------

    // worker.c

    typedef enum { IDLE = 0, WORKING = 1 } State;

    typedef struct {
        int id;        // campo 0
        State state;   // campo 1
    } Worker;

    // Tomo una tarea
    // ABSTRAKTOR_FUNC: w->1
    void take_task(Worker* w) {
        if (w->state == IDLE) {
            w->state = WORKING;
        }
    }

    // Termino la tarea
    // ABSTRAKTOR_FUNC: w->1
    void finish_task(Worker* w) {
        // BUG: no verifica que state == WORKING
        // Debería ser: if (w->state == WORKING)
        w->state = IDLE;
    }


EL BUG
------

    void finish_task(Worker* w) {
        // BUG: se ejecuta siempre, sin verificar estado
        w->state = IDLE;  // <-- debería verificar primero
    }


ESCENARIO DEL BUG
-----------------

    Tiempo | Coordinator              | Worker                    | Red
    -------|--------------------------|---------------------------|-----
     t1    |                          | state=IDLE                |
     t2    | send(take_task)          |                           | ok
     t3    |                          | take_task()               |
           |                          | state: IDLE->WORKING      |
     t4    |                          | [procesando...]           |
     t5    |                          | finish_task()             |
           |                          | state: WORKING->IDLE      |
     t6    | send(finish_task)        | [mensaje duplicado        | delay
           | (retry por timeout)      |  o retry]                 |
     t7    |                          | finish_task() <-- OTRA VEZ|
           |                          | state: IDLE->IDLE         |
           |                          | (no crashea, pero...)     |

Problema: El worker ejecuta finish_task cuando ya está IDLE. No crashea, 
pero indica que el worker "terminó" una tarea que no tenía.


EPA ESPERADA (spec correcta)
----------------------------

           take_task
        ----------------->
       |                  |
    [state=0]         [state=1]
     (IDLE)           (WORKING)
       |                  |
        <-----------------
           finish_task

- take_task: solo desde IDLE
- finish_task: solo desde WORKING


EPA GENERADA POR ABSTRAKTOR (con bug)
-------------------------------------

           take_task
        ----------------->
       |                  |
    [state=0]         [state=1]
     (IDLE) <----+    (WORKING)
       |         |        |
       | finish_ |        |
       | task !! |        |
       +---------+        |
             |            |
             +------------+
               finish_task

Problema visible: Hay una transición finish_task desde state=0 (IDLE) 
que NO DEBERÍA EXISTIR.


COMPARACIÓN: EPA spec vs EPA generada
-------------------------------------

    Transición    | Desde IDLE      | Desde WORKING
    --------------|-----------------|---------------
    take_task     | ok esperado     | no ocurre
    finish_task   | !! SÍ ocurre    | ok esperado

La transición IDLE --finish_task--> IDLE no debería existir.


POR QUÉ TESTING TRADICIONAL NO LO DETECTA
-----------------------------------------

    void test_finish_task() {
        Worker w = {.id = 1, .state = WORKING};
        finish_task(&w);
        assert(w.state == IDLE);  // PASA
    }

El test prueba el caso normal (finish desde WORKING). No prueba qué pasa 
si llega un finish_task duplicado cuando ya está IDLE.


VERSIÓN CORREGIDA
-----------------

    void finish_task(Worker* w) {
        if (w->state == WORKING) {  // <-- agrega verificación
            w->state = IDLE;
        }
    }

Con esta corrección, la EPA ya no muestra la transición IDLE --finish_task--> IDLE.


RESUMEN
-------

1. Bug: finish_task no verifica el estado antes de ejecutarse
2. Causa: Mensajes duplicados/retrasados por condiciones de red
3. Testing tradicional: No lo detecta porque prueba el caso feliz
4. EPA detecta: Muestra transición IDLE -> IDLE via finish_task que no debería existir
5. Mallory: Genera las condiciones de red que producen el bug
