#include "quota_ui.h"
#include "lvgl.h"
#include <stdio.h>
#include <string.h>
LV_FONT_DECLARE(cq_font_22);
LV_FONT_DECLARE(cq_font_18);
LV_FONT_DECLARE(cq_font_60);
static lv_obj_t *sync_label,*metric_label,*value_label,*reset_label,*footer;
static lv_obj_t *outer_arc,*inner_arc,*tabs[2],*tab_labels[2];
static lv_obj_t *rows[2],*row_labels[2][3],*separator;
static cq_view_t last_view;
static uint8_t requested_metric;
static volatile uint8_t displayed_metric;
static cq_layout_t layout=CQ_LAYOUT_FOCUS;
static lv_obj_t *content;

static lv_obj_t *label(int x,int y,int width,const lv_font_t *font) {
    lv_obj_t *obj=lv_label_create(content);lv_obj_set_pos(obj,x,y);lv_obj_set_width(obj,width);
    lv_label_set_long_mode(obj,LV_LABEL_LONG_CLIP);lv_obj_set_style_text_align(obj,LV_TEXT_ALIGN_CENTER,0);
    lv_obj_set_style_text_font(obj,font,0);lv_obj_set_style_text_color(obj,lv_color_hex(0xB3C0C5),0);
    return obj;
}
static lv_obj_t *arc(int x,int y,int size) {
    lv_obj_t *obj=lv_arc_create(content);lv_obj_set_pos(obj,x,y);lv_obj_set_size(obj,size,size);
    lv_arc_set_range(obj,0,100);lv_arc_set_rotation(obj,270);lv_arc_set_bg_angles(obj,0,360);
    lv_obj_remove_style(obj,NULL,LV_PART_KNOB);lv_obj_clear_flag(obj,LV_OBJ_FLAG_CLICKABLE);
    lv_obj_set_style_arc_width(obj,20,LV_PART_MAIN);lv_obj_set_style_arc_width(obj,20,LV_PART_INDICATOR);
    lv_obj_set_style_arc_color(obj,lv_color_hex(0x27373E),LV_PART_MAIN);
    lv_obj_set_style_arc_rounded(obj,true,LV_PART_INDICATOR);
    return obj;
}
static void update_metric(void);
static void choose_metric(lv_event_t *event) {
    requested_metric=(uint8_t)(uintptr_t)lv_event_get_user_data(event);update_metric();
}
void cq_ui_init(void) {
    lv_obj_t *screen=lv_scr_act();lv_obj_clear_flag(screen,LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_style_bg_color(screen,lv_color_black(),0);
    lv_obj_set_style_bg_opa(screen,LV_OPA_COVER,0);
    content=lv_obj_create(screen);lv_obj_remove_style_all(content);
    lv_obj_set_size(content,390,450);lv_obj_set_pos(content,0,0);
    lv_obj_clear_flag(content,LV_OBJ_FLAG_SCROLLABLE|LV_OBJ_FLAG_CLICKABLE);
    lv_obj_t *title=label(24,16,174,&cq_font_22);lv_obj_set_style_text_align(title,LV_TEXT_ALIGN_LEFT,0);
    lv_obj_set_style_text_color(title,lv_color_hex(0xF2F6F7),0);lv_label_set_text(title,"Codex额度");
    sync_label=label(216,16,150,&cq_font_18);
    lv_obj_set_style_bg_color(sync_label,lv_color_hex(0x202E34),0);lv_obj_set_style_bg_opa(sync_label,LV_OPA_COVER,0);
    lv_obj_set_style_radius(sync_label,16,0);lv_obj_set_style_pad_ver(sync_label,4,0);
    /* Same center and 28 px radial spacing as the accepted concentric preview. */
    outer_arc=arc(63,74,264);inner_arc=arc(91,102,208);
    metric_label=label(110,172,170,&cq_font_18);
    value_label=label(100,186,190,&cq_font_60);
    for(uint8_t i=0;i<2;i++) {
        tabs[i]=lv_btn_create(content);lv_obj_set_pos(tabs[i],24+i*176,345);lv_obj_set_size(tabs[i],166,44);
        lv_obj_set_style_radius(tabs[i],12,0);lv_obj_set_style_shadow_width(tabs[i],0,0);
        lv_obj_add_event_cb(tabs[i],choose_metric,LV_EVENT_CLICKED,(void *)(uintptr_t)i);
        tab_labels[i]=lv_label_create(tabs[i]);lv_obj_set_style_text_font(tab_labels[i],&cq_font_18,0);
        lv_obj_center(tab_labels[i]);
        rows[i]=lv_btn_create(content);lv_obj_set_pos(rows[i],24,313+i*52);lv_obj_set_size(rows[i],342,44);
        lv_obj_set_style_bg_opa(rows[i],LV_OPA_TRANSP,0);lv_obj_set_style_border_width(rows[i],0,0);
        lv_obj_set_style_shadow_width(rows[i],0,0);lv_obj_set_style_pad_all(rows[i],0,0);
        lv_obj_add_event_cb(rows[i],choose_metric,LV_EVENT_CLICKED,(void *)(uintptr_t)i);
        for(uint8_t j=0;j<3;j++) {
            row_labels[i][j]=lv_label_create(rows[i]);
            lv_label_set_long_mode(row_labels[i][j],LV_LABEL_LONG_CLIP);
            lv_obj_set_pos(row_labels[i][j],j==0?0:j==1?119:213,j==1?2:8);
            lv_obj_set_width(row_labels[i][j],j==0?110:j==1?82:129);
            lv_obj_set_style_text_font(row_labels[i][j],j==1?&lv_font_montserrat_28:&cq_font_18,0);
            lv_obj_set_style_text_color(row_labels[i][j],lv_color_hex(0xB3C0C5),0);
            lv_obj_set_style_text_align(row_labels[i][j],j==2?LV_TEXT_ALIGN_RIGHT:LV_TEXT_ALIGN_LEFT,0);
        }
    }
    separator=lv_obj_create(content);lv_obj_set_pos(separator,24,355);lv_obj_set_size(separator,342,1);
    lv_obj_set_style_border_width(separator,0,0);lv_obj_set_style_pad_all(separator,0,0);
    lv_obj_set_style_bg_color(separator,lv_color_hex(0x27373E),0);lv_obj_clear_flag(separator,LV_OBJ_FLAG_CLICKABLE);
    reset_label=label(24,392,342,&cq_font_18);footer=label(24,421,342,&cq_font_18);
}
static void show_arc(lv_obj_t *obj,uint8_t percent) {
    lv_arc_set_value(obj,percent<=100?percent:0);
    lv_obj_set_style_arc_color(obj,lv_color_hex(cq_quota_color(percent)),LV_PART_INDICATOR);
    lv_obj_set_style_arc_opa(obj,percent<=100&&percent>0?LV_OPA_COVER:LV_OPA_TRANSP,LV_PART_INDICATOR);
}
static void update_metric(void) {
    const cq_view_t *v=&last_view;bool dual=cq_dual_layout(v);
    bool details=layout==CQ_LAYOUT_ROWS;
    uint8_t metric=cq_active_metric(v,requested_metric);displayed_metric=metric;
    uint8_t percent=metric?v->weekly:v->five;uint32_t reset=metric?v->weekly_reset:v->five_reset;
    lv_obj_set_pos(outer_arc,details?80:63,details?77:86);lv_obj_set_size(outer_arc,details?230:264,details?230:264);
    lv_obj_set_pos(inner_arc,details?108:91,details?105:114);lv_obj_set_size(inner_arc,details?174:208,details?174:208);
    lv_arc_set_rotation(outer_arc,details?140:270);lv_arc_set_rotation(inner_arc,details?140:270);
    lv_arc_set_bg_angles(outer_arc,0,details?300:360);lv_arc_set_bg_angles(inner_arc,0,details?300:360);
    lv_obj_set_y(metric_label,details?160:172);
    lv_obj_set_y(value_label,details?(percent==100?181:160):(percent==100?203:182));
    show_arc(outer_arc,dual?v->five:percent);
    if(dual){lv_obj_clear_flag(inner_arc,LV_OBJ_FLAG_HIDDEN);show_arc(inner_arc,v->weekly);}
    else lv_obj_add_flag(inner_arc,LV_OBJ_FLAG_HIDDEN);
    lv_label_set_text(metric_label,metric?"周额度":"5小时额度");
    char text[40];if(percent>100)strcpy(text,"--");else snprintf(text,sizeof(text),"%u%%",percent);
    lv_obj_set_style_text_font(value_label,percent==100?&lv_font_montserrat_48:&cq_font_60,0);
    lv_label_set_text(value_label,text);lv_obj_set_style_text_color(value_label,lv_color_hex(cq_quota_color(percent)),0);
    for(uint8_t i=0;i<2;i++) {
        if(dual&&!details)lv_obj_clear_flag(tabs[i],LV_OBJ_FLAG_HIDDEN);else lv_obj_add_flag(tabs[i],LV_OBJ_FLAG_HIDDEN);
        if(dual&&details)lv_obj_clear_flag(rows[i],LV_OBJ_FLAG_HIDDEN);else lv_obj_add_flag(rows[i],LV_OBJ_FLAG_HIDDEN);
        uint8_t p=i?v->weekly:v->five;
        if(p>100)snprintf(text,sizeof(text),"%s --",i?"周额度":"5小时");
        else snprintf(text,sizeof(text),"%s %u%%",i?"周额度":"5小时",p);
        lv_label_set_text(tab_labels[i],text);
        lv_obj_set_style_bg_color(tabs[i],lv_color_hex(i==metric?0x27373E:0x000000),0);
        lv_obj_set_style_text_color(tab_labels[i],lv_color_hex(0xF2F6F7),0);
        lv_label_set_text(row_labels[i][0],i?"周额度":"5小时额度");
        if(p>100)strcpy(text,"--");else snprintf(text,sizeof(text),"%u%%",p);
        lv_label_set_text(row_labels[i][1],text);
        lv_obj_set_style_text_color(row_labels[i][1],lv_color_hex(cq_quota_color(p)),0);
        uint32_t row_reset=i?v->weekly_reset:v->five_reset;char row_duration[24];
        cq_reset_text(row_duration,sizeof(row_duration),v->now,row_reset,i!=0);
        if(row_reset==0)strcpy(text,"时间未知");else if(row_reset<=v->now)strcpy(text,"等待更新");
        else snprintf(text,sizeof(text),"重置 %s",row_duration);
        lv_label_set_text(row_labels[i][2],text);
    }
    if(dual&&details)lv_obj_clear_flag(separator,LV_OBJ_FLAG_HIDDEN);else lv_obj_add_flag(separator,LV_OBJ_FLAG_HIDDEN);
    char duration[24];cq_reset_text(duration,sizeof(duration),v->now,reset,metric!=0);
    if(reset==0)strcpy(text,"时间未知");else if(reset<=v->now)strcpy(text,"等待更新");
    else snprintf(text,sizeof(text),"重置 %s",duration);
    lv_label_set_text(reset_label,text);
    lv_obj_set_y(reset_label,details?334:262);
    if(details&&dual)lv_obj_add_flag(reset_label,LV_OBJ_FLAG_HIDDEN);else lv_obj_clear_flag(reset_label,LV_OBJ_FLAG_HIDDEN);
}
void cq_ui_render(const cq_view_t *view,bool pin_visible,uint32_t pin,uint8_t ble_phase) {
    last_view=*view;update_metric();
    char text[48];uint32_t color=0xB3C0C5;
    if(view->status==CQ_SYNCED){strcpy(text,"已同步");color=0x72D6A6;}
    else if(view->status==CQ_NO_DATA)strcpy(text,"等待同步");
    else {
        const char *prefix=view->status==CQ_OFFLINE?"离线":"缓存";color=view->status==CQ_OFFLINE?0xFFB4AB:0xFFC968;
        if(view->age==UINT64_MAX)snprintf(text,sizeof(text),"%s",prefix);
        else if(view->age<3600)snprintf(text,sizeof(text),"%s %lu分",prefix,(unsigned long)(view->age/60));
        else if(view->age<86400)snprintf(text,sizeof(text),"%s %lu小时",prefix,(unsigned long)(view->age/3600));
        else snprintf(text,sizeof(text),"%s %lu天",prefix,(unsigned long)(view->age/86400>999?999:view->age/86400));
    }
    lv_label_set_text(sync_label,text);lv_obj_set_style_text_color(sync_label,lv_color_hex(color),0);
    if(pin_visible)snprintf(text,sizeof(text),"配对码 %06lu",(unsigned long)pin);
    else snprintf(text,sizeof(text),"%s",ble_phase==9?"蓝牙不可用":ble_phase<3?"蓝牙启动中":ble_phase==4?"蓝牙已连接":"等待电脑蓝牙");
    lv_label_set_text(footer,text);
}
uint8_t cq_ui_metric(void) {return displayed_metric;}
void cq_ui_toggle_layout(void) {layout=cq_next_layout(layout);update_metric();}
uint8_t cq_ui_layout(void) {return (uint8_t)layout;}
void cq_ui_screen(bool visible,int8_t dx,int8_t dy) {
    lv_obj_set_pos(content,dx,dy);
    if(visible)lv_obj_clear_flag(content,LV_OBJ_FLAG_HIDDEN);
    else lv_obj_add_flag(content,LV_OBJ_FLAG_HIDDEN);
}
