#ifndef CQ_QUOTA_STATE_H
#define CQ_QUOTA_STATE_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#define CQ_PACKET_LEN 20
#define CQ_UNKNOWN 255
#define CQ_NOT_APPLICABLE 254
#define CQ_LINK_TIMEOUT 45
#define CQ_FRESH_SECONDS 120
typedef enum { CQ_NO_DATA, CQ_SYNCED, CQ_CACHED, CQ_OFFLINE } cq_status_t;
typedef struct {
    bool has_packet, connected;
    uint8_t freshness, five, weekly;
    uint32_t sent, confirmed, five_reset, weekly_reset;
    uint64_t received_at;
} cq_state_t;
typedef struct {
    cq_status_t status;
    uint8_t five, weekly;
    uint32_t five_reset, weekly_reset;
    uint64_t now, age;
} cq_view_t;
bool cq_receive(cq_state_t *state, const uint8_t *packet, size_t length, uint64_t uptime);
void cq_disconnect(cq_state_t *state);
void cq_reset_text(char *text,size_t size,uint64_t now,uint32_t reset,bool weekly);
void cq_view(const cq_state_t *state, uint64_t uptime, cq_view_t *view);
bool cq_dual_layout(const cq_view_t *view);
uint8_t cq_active_metric(const cq_view_t *view,uint8_t requested);
uint32_t cq_quota_color(uint8_t value);
typedef enum { CQ_LAYOUT_FOCUS, CQ_LAYOUT_ROWS } cq_layout_t;
cq_layout_t cq_next_layout(cq_layout_t layout);
#define CQ_SCREEN_IDLE_SECONDS 180
#define CQ_SCREEN_BRIGHTNESS 30
typedef struct { bool offline_tracking; uint64_t offline_since; } cq_screen_t;
bool cq_screen_update(cq_screen_t *screen,uint64_t now,bool connected,bool pairing,bool activity);
void cq_screen_shift(uint64_t now,int8_t *dx,int8_t *dy);
#endif
