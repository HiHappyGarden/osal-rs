/**
 * Event implementation for FreeRTOS POSIX port
 * Required by ThirdParty/GCC/Posix port
 */

#include <pthread.h>
#include <stdlib.h>
#include <errno.h>

typedef struct Event {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    int signaled;
} Event_t;

/**
 * Create an event
 */
void* event_create(void) {
    Event_t* event = (Event_t*)malloc(sizeof(Event_t));
    if (event == NULL) {
        return NULL;
    }

    pthread_mutex_init(&event->mutex, NULL);
    pthread_cond_init(&event->cond, NULL);
    event->signaled = 0;

    return event;
}

/**
 * Wait for an event to be signaled
 */
void event_wait(void* event_handle) {
    if (event_handle == NULL) {
        return;
    }

    Event_t* event = (Event_t*)event_handle;

    pthread_mutex_lock(&event->mutex);
    while (!event->signaled) {
        pthread_cond_wait(&event->cond, &event->mutex);
    }
    event->signaled = 0;
    pthread_mutex_unlock(&event->mutex);
}

/**
 * Signal an event
 */
void event_signal(void* event_handle) {
    if (event_handle == NULL) {
        return;
    }

    Event_t* event = (Event_t*)event_handle;

    pthread_mutex_lock(&event->mutex);
    event->signaled = 1;
    pthread_cond_signal(&event->cond);
    pthread_mutex_unlock(&event->mutex);
}

/**
 * Delete an event
 */
void event_delete(void* event_handle) {
    if (event_handle == NULL) {
        return;
    }

    Event_t* event = (Event_t*)event_handle;

    pthread_mutex_destroy(&event->mutex);
    pthread_cond_destroy(&event->cond);
    free(event);
}

