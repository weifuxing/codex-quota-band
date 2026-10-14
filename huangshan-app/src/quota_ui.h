#ifndef CQ_QUOTA_UI_H
#define CQ_QUOTA_UI_H
#include "quota_state.h"
void cq_ui_init(void);
void cq_ui_render(const cq_view_t *view,bool pin_visible,uint32_t pin,uint8_t ble_phase);
uint8_t cq_ui_metric(void);
void cq_ui_toggle_layout(void);
uint8_t cq_ui_layout(void);
void cq_ui_screen(bool visible,int8_t dx,int8_t dy);
#endif
