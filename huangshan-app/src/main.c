/* CodexQuota Huangshan board firmware. BLE API usage follows SiFli's
 * Apache-2.0 peripheral and pair examples; application code uses project MIT. */
#include <rtthread.h>
#include <rtdevice.h>
#include <board.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include "littlevgl2rtt.h"
#include "lvgl.h"
#include "lv_ex_data.h"
#include "bf0_ble_gap.h"
#include "bf0_sibles.h"
#include "bf0_sibles_advertising.h"
#include "ble_connection_manager.h"
#include "quota_state.h"
#include "quota_ui.h"
#include "button.h"
#define CQ_LAYOUT_BUTTON_MSG 0x10000u
#define CQ_TOUCH_MSG 0x10001u
extern lv_indev_t *touch_get_indev_handler(void);

#define SERIAL_UUID_16(x) {((uint8_t)((x)&0xff)),((uint8_t)((x)>>8))}
#define CQ_SERVICE_UUID {0x10,0x5f,0xb2,0xa1,0xd6,0x58,0x40,0xa1,0x8b,0x4d,0xca,0x37,0x90,0xc7,0x84,0xf9}
#define CQ_QUOTA_UUID   {0x10,0x5f,0xb2,0xa1,0xd6,0x58,0x40,0xa1,0x8b,0x4d,0xca,0x37,0x91,0xc7,0x84,0xf9}
enum {CQ_SERVICE,CQ_CHAR,CQ_VALUE,CQ_ATT_COUNT};
static uint8_t service_uuid[16]=CQ_SERVICE_UUID;
BLE_GATT_SERVICE_DEFINE_128(quota_att_db) {
    BLE_GATT_SERVICE_DECLARE(CQ_SERVICE,SERIAL_UUID_16_PRI_SERVICE,BLE_GATT_PERM_READ_ENABLE),
    BLE_GATT_CHAR_DECLARE(CQ_CHAR,SERIAL_UUID_16_CHARACTERISTIC,BLE_GATT_PERM_READ_ENABLE),
    BLE_GATT_CHAR_VALUE_DECLARE(CQ_VALUE,CQ_QUOTA_UUID,
        BLE_GATT_PERM_WRITE_REQ_ENABLE|BLE_GATT_PERM_WRITE_PERMISSION_AUTH,
        BLE_GATT_VALUE_PERM_UUID_128,CQ_PACKET_LEN),
};
SIBLES_ADVERTISING_CONTEXT_DECLAR(quota_advertising);

static cq_state_t state;
static uint64_t uptime_seconds;
static rt_mailbox_t mailbox;
static bool pairing_pin_visible;
static uint32_t pairing_pin;
static uint32_t accepted_packets;
static uint8_t ble_phase,ble_error;
static uint32_t layout_clicks;
static cq_screen_t screen_policy;
static bool screen_on=true;
static uint8_t screen_brightness=CQ_SCREEN_BRIGHTNESS;
static int8_t shift_x,shift_y;
static rt_device_t lcd_device;
static rt_err_t (*original_touch_rx)(rt_device_t,rt_size_t);
static volatile bool touch_pending;

static rt_err_t touch_rx(rt_device_t device,rt_size_t size) {
    rt_err_t result=original_touch_rx?original_touch_rx(device,size):RT_EOK;
    rt_enter_critical();
    if(!touch_pending&&mailbox&&rt_mb_send(mailbox,CQ_TOUCH_MSG)==RT_EOK)touch_pending=true;
    rt_exit_critical();
    return result;
}
static int touch_wake_init(void) {
    rt_device_t touch=rt_device_find("touch");
    if(!touch||!touch->rx_indicate||!touch_get_indev_handler())return -RT_ERROR;
    rt_enter_critical();original_touch_rx=touch->rx_indicate;touch->rx_indicate=touch_rx;rt_exit_critical();
    return RT_EOK;
}
static void set_screen_brightness(uint8_t brightness) {
    if(rt_device_control(lcd_device,RTGRAPHIC_CTRL_SET_BRIGHTNESS,&brightness)==RT_EOK)
        screen_brightness=brightness;
}

static void layout_button_event(int32_t pin,button_action_t action) {
    if(pin==BSP_KEY2_PIN&&action==BUTTON_CLICKED&&mailbox) {
        if(rt_mb_send(mailbox,CQ_LAYOUT_BUTTON_MSG)==RT_EOK) {
            rt_enter_critical();layout_clicks++;rt_exit_critical();
        }
    }
}
static int layout_button_init(void) {
    button_cfg_t cfg={0};cfg.pin=BSP_KEY2_PIN;cfg.active_state=BUTTON_ACTIVE_HIGH;
    cfg.mode=PIN_MODE_INPUT;cfg.debounce_time=30;cfg.button_handler=layout_button_event;
    int32_t id=button_init(&cfg);return id<0?-RT_ERROR:button_enable(id);
}



static uint8_t *get_value(uint8_t conn_idx,uint8_t idx,uint16_t *len) { *len=0;return NULL; }
static uint8_t set_value(uint8_t conn_idx,sibles_set_cbk_t *p) {
    if(p->idx!=CQ_VALUE)return SIBLES_ATT_ERR_WRITE_NOT_PERMITTED;
    rt_enter_critical();
    bool accepted=cq_receive(&state,p->value,p->len,uptime_seconds);
    if(accepted){pairing_pin_visible=false;accepted_packets++;}
    rt_exit_critical();
    return accepted?0:SIBLES_ATT_ERR_INVALID_ATTRIBUTE_VAL_LEN;
}
static uint8_t advertising_event(uint8_t event,void *context,void *data) {
    if(event==SIBLES_ADV_EVT_ADV_STARTED) {
        sibles_adv_evt_startted_t *status=(sibles_adv_evt_startted_t *)data;
        ble_error=status->status;ble_phase=status->status==0?3:9;
    }
    return 0;
}
// The SDK peripheral example enables DIS during host startup.
uint8_t ble_app_dis_enable(void) {return 1;}
uint8_t sibles_advertising_disc_mode_get(void) {return GAPM_ADV_MODE_GEN_DISC;}
static int start_advertising(void) {
    const char name[]="CodexQuota-HSPI";
    sibles_advertising_para_t p={0};
    p.own_addr_type=GAPM_STATIC_ADDR;
    p.config.adv_mode=SIBLES_ADV_CONNECT_MODE;
    p.config.mode_config.conn_config.duration=0;
    p.config.mode_config.conn_config.interval=0xA0;
    p.adv_data.disc_mode=GAPM_ADV_MODE_GEN_DISC;
    p.config.max_tx_pwr=0x7F;p.config.is_auto_restart=1;
    p.evt_handler=advertising_event;
    ble_gap_dev_name_t *device_name=rt_malloc(sizeof(ble_gap_dev_name_t)+sizeof(name)-1);
    if(!device_name)return -RT_ENOMEM;
    device_name->len=sizeof(name)-1;memcpy(device_name->name,name,device_name->len);
    ble_gap_set_dev_name(device_name);rt_free(device_name);
    p.rsp_data.completed_name=rt_malloc(sizeof(sibles_adv_type_name_t)+sizeof(name)-1);
    p.adv_data.completed_uuid=rt_calloc(1,sizeof(sibles_adv_type_srv_uuid_t)+sizeof(sibles_adv_uuid_t));
    if(!p.rsp_data.completed_name||!p.adv_data.completed_uuid) {
        rt_free(p.rsp_data.completed_name);rt_free(p.adv_data.completed_uuid);return -RT_ENOMEM;
    }
    p.rsp_data.completed_name->name_len=sizeof(name)-1;
    memcpy(p.rsp_data.completed_name->name,name,sizeof(name)-1);
    p.adv_data.completed_uuid->count=1;p.adv_data.completed_uuid->uuid_list[0].uuid_len=ATT_UUID_128_LEN;
    memcpy(p.adv_data.completed_uuid->uuid_list[0].uuid.uuid_128,service_uuid,16);
    int result=sibles_advertising_init(quota_advertising,&p);
    rt_free(p.rsp_data.completed_name);rt_free(p.adv_data.completed_uuid);
    if(result==SIBLES_ADV_NO_ERR)result=sibles_advertising_start(quota_advertising);
    return result;
}
static int bluetooth_ready(void) {
    BLE_GATT_SERVICE_INIT_128(svc,quota_att_db,CQ_ATT_COUNT,
        BLE_GATT_SERVICE_PERM_NOAUTH|BLE_GATT_SERVICE_PERM_UUID_128,service_uuid);
    sibles_hdl handle=sibles_register_svc_128(&svc);
    if(!handle){ble_phase=9;ble_error=1;return -RT_ERROR;}
    ble_phase=2;
    sibles_register_cbk(handle,get_value,set_value);
    connection_manager_set_bond_ack(BOND_PENDING);
    connection_manager_set_bond_cnf_iocap(GAP_IO_CAP_DISPLAY_ONLY);
    connection_manager_set_bond_cnf_auth(GAP_AUTH_REQ_SEC_CON_MITM_BOND);
    connection_manager_set_connected_auth(GAP_AUTH_REQ_SEC_CON_MITM_BOND);
    return start_advertising();
}
int quota_ble_event(uint16_t event,uint8_t *data,uint16_t len,uint32_t context) {
    if(event==BLE_POWER_ON_IND) {ble_phase=1;if(mailbox)rt_mb_send(mailbox,event);}
    else if(event==BLE_GAP_CONNECTED_IND) {ble_phase=4;}
    else if(event==BLE_GAP_DISCONNECTED_IND) {
        rt_enter_critical();cq_disconnect(&state);pairing_pin_visible=false;rt_exit_critical();
    }
    else if(event==CONNECTION_MANAGER_BOND_AUTH_INFOR) {
        connection_manager_bond_ack_infor_t *info=(connection_manager_bond_ack_infor_t *)data;
        if(info->request==GAPC_PAIRING_REQ)connection_manager_bond_ack_reply(info->conn_idx,info->request,true);
        else if(info->request==GAPC_TK_EXCH&&info->type==GAP_TK_DISPLAY) {
            rt_enter_critical();pairing_pin=info->confirm_data;pairing_pin_visible=true;rt_exit_critical();
            connection_manager_bond_ack_reply(info->conn_idx,info->request,true);
        } else connection_manager_bond_ack_reply(info->conn_idx,info->request,false);
    }
    return 0;
}
BLE_EVENT_REGISTER(quota_ble_event,NULL);
#ifndef NVDS_AUTO_UPDATE_MAC_ADDRESS_ENABLE
ble_common_update_type_t ble_request_public_address(bd_addr_t *addr) {
    return bt_mac_addr_generate_via_uid_v2(addr)==0?BLE_UPDATE_ONCE:BLE_UPDATE_NO_UPDATE;
}
#endif

static void render(void) {
    cq_view_t view;bool pin_visible;uint32_t pin;uint8_t phase;
    rt_enter_critical();cq_view(&state,uptime_seconds,&view);pin_visible=pairing_pin_visible;pin=pairing_pin;phase=ble_phase;rt_exit_critical();
    cq_ui_render(&view,pin_visible,pin,phase);
}
int main(void) {
    mailbox=rt_mb_create("cq_ble",4,RT_IPC_FLAG_FIFO);if(!mailbox)return -RT_ENOMEM;
    int result=littlevgl2rtt_init("lcd");if(result!=RT_EOK)return result;
    lv_ex_data_pool_init();cq_ui_init();render();
    lcd_device=rt_device_find("lcd");if(!lcd_device)return -RT_ERROR;
    set_screen_brightness(CQ_SCREEN_BRIGHTNESS);
    if(touch_wake_init()!=RT_EOK)return -RT_ERROR;
    if(layout_button_init()!=0)return -RT_ERROR;
    sifli_ble_enable();
    rt_tick_t last_tick=rt_tick_get();uint64_t ticks=0,last_render=UINT64_MAX;
    while(1) {
        rt_tick_t tick=rt_tick_get();ticks+=(rt_tick_t)(tick-last_tick);last_tick=tick;
        rt_enter_critical();uptime_seconds=ticks/RT_TICK_PER_SECOND;rt_exit_critical();
        bool activity=false;
        rt_ubase_t event;if(rt_mb_recv(mailbox,&event,0)==RT_EOK) {
            if(event==BLE_POWER_ON_IND){if(bluetooth_ready()!=0)ble_phase=9;}
            else if(event==CQ_LAYOUT_BUTTON_MSG) {
                activity=true;if(screen_on)cq_ui_toggle_layout();
            } else if(event==CQ_TOUCH_MSG) {
                rt_enter_critical();touch_pending=false;rt_exit_critical();activity=true;
                if(!screen_on)lv_indev_wait_release(touch_get_indev_handler());
            }
        }
        bool connected,pairing;
        rt_enter_critical();
        connected=state.connected&&uptime_seconds-state.received_at<CQ_LINK_TIMEOUT;
        pairing=pairing_pin_visible;
        rt_exit_critical();
        bool visible=cq_screen_update(&screen_policy,uptime_seconds,connected,pairing,activity);
        if(visible!=screen_on) {
            screen_on=visible;cq_ui_screen(visible,shift_x,shift_y);
            if(visible) {
                lv_indev_wait_release(touch_get_indev_handler());
                render();lv_obj_invalidate(lv_scr_act());
            }
            /* Keep the shared touch supply alive: never use LCD POWEROFF here. */
            set_screen_brightness(visible?CQ_SCREEN_BRIGHTNESS:0);
        }
        if(screen_on&&uptime_seconds!=last_render) {
            cq_screen_shift(uptime_seconds,&shift_x,&shift_y);
            cq_ui_screen(true,shift_x,shift_y);render();last_render=uptime_seconds;
        }
        lv_task_handler();rt_thread_mdelay(10);
    }
}

#ifdef RT_USING_FINSH
static void cq_status(void) {
    cq_view_t v;uint32_t count;uint32_t confirmed;
    rt_enter_critical();cq_view(&state,uptime_seconds,&v);count=accepted_packets;confirmed=state.confirmed;rt_exit_critical();
    rt_kprintf("CQ_STATUS state=%u five=%u weekly=%u packets=%lu confirmed=%lu ble=%u error=%u layout=%u clicks=%lu\n",(unsigned)v.status,(unsigned)v.five,(unsigned)v.weekly,(unsigned long)count,(unsigned long)confirmed,(unsigned)ble_phase,(unsigned)ble_error,(unsigned)cq_ui_layout(),(unsigned long)layout_clicks);
    rt_kprintf("CQ_SCREEN on=%u brightness=%u shift=%d,%d idle=%lu\n",(unsigned)screen_on,(unsigned)screen_brightness,(int)shift_x,(int)shift_y,(unsigned long)(screen_policy.offline_tracking?uptime_seconds-screen_policy.offline_since:0));
}
MSH_CMD_EXPORT(cq_status, Display privacy-bounded quota status.);
#endif
