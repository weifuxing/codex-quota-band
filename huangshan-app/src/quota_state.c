#include "quota_state.h"

bool cq_screen_update(cq_screen_t *screen,uint64_t now,bool connected,bool pairing,bool activity) {
    if(connected||pairing) {screen->offline_tracking=false;return true;}
    if(!screen->offline_tracking||activity) {
        screen->offline_tracking=true;screen->offline_since=now;
    }
    return now-screen->offline_since<CQ_SCREEN_IDLE_SECONDS;
}
void cq_screen_shift(uint64_t now,int8_t *dx,int8_t *dy) {
    static const int8_t points[9][2]={{0,0},{-2,-2},{0,-2},{2,-2},{2,0},{2,2},{0,2},{-2,2},{-2,0}};
    uint8_t index=(uint8_t)((now/60)%9);*dx=points[index][0];*dy=points[index][1];
}
#include <string.h>
#include <stdio.h>
static uint32_t get32(const uint8_t *p) { return (uint32_t)p[0]|((uint32_t)p[1]<<8)|((uint32_t)p[2]<<16)|((uint32_t)p[3]<<24); }
bool cq_receive(cq_state_t *s,const uint8_t *p,size_t length,uint64_t uptime) {
    if(!p||length!=CQ_PACKET_LEN||p[0]!=2||p[1]>2||
       (p[2]>100&&p[2]!=CQ_UNKNOWN&&p[2]!=CQ_NOT_APPLICABLE)||(p[3]>100&&p[3]!=CQ_UNKNOWN&&p[3]!=CQ_NOT_APPLICABLE))return false;
    uint32_t sent=get32(p+4),confirmed=get32(p+8);
    if(sent==0||confirmed>sent||(s->has_packet&&sent<=s->sent))return false;
    if(p[1]==1&&(confirmed==0||sent-confirmed>CQ_FRESH_SECONDS))return false;
    if((p[1]==0)!=(p[2]>100&&p[3]>100))return false;
    if((p[2]<=100&&get32(p+12)==0)||(p[3]<=100&&get32(p+16)==0))return false;
    bool retain=p[1]==0&&p[2]==CQ_UNKNOWN&&p[3]==CQ_UNKNOWN&&s->has_packet&&(s->five<=100||s->weekly<=100);
    uint8_t five=p[2]==CQ_UNKNOWN&&s->five==CQ_NOT_APPLICABLE?CQ_NOT_APPLICABLE:p[2];
    uint8_t weekly=p[3]==CQ_UNKNOWN&&s->weekly==CQ_NOT_APPLICABLE?CQ_NOT_APPLICABLE:p[3];
    if(retain) {
        s->connected=true;s->freshness=2;s->sent=sent;s->received_at=uptime;return true;
    }
    s->has_packet=true;s->connected=true;s->freshness=p[1];s->five=five;s->weekly=weekly;
    s->sent=sent;s->confirmed=confirmed;s->five_reset=get32(p+12);s->weekly_reset=get32(p+16);s->received_at=uptime;
    return true;
}
void cq_disconnect(cq_state_t *s) { s->connected=false; }
void cq_view(const cq_state_t *s,uint64_t uptime,cq_view_t *v) {
    memset(v,0,sizeof(*v));v->five=v->weekly=CQ_UNKNOWN;v->status=CQ_NO_DATA;
    if(!s->has_packet)return;
    uint64_t elapsed=uptime>=s->received_at?uptime-s->received_at:0;
    v->now=(uint64_t)s->sent+elapsed;v->age=s->confirmed?v->now-s->confirmed:UINT64_MAX;
    v->five_reset=s->five_reset;v->weekly_reset=s->weekly_reset;
    v->five=s->five==CQ_NOT_APPLICABLE?CQ_NOT_APPLICABLE:s->five_reset>v->now?s->five:CQ_UNKNOWN;v->weekly=s->weekly==CQ_NOT_APPLICABLE?CQ_NOT_APPLICABLE:s->weekly_reset>v->now?s->weekly:CQ_UNKNOWN;
    if(!s->connected||elapsed>=CQ_LINK_TIMEOUT)v->status=CQ_OFFLINE;
    else if(s->freshness==0)v->status=CQ_NO_DATA;
    else if(s->freshness==1&&v->age<=CQ_FRESH_SECONDS)v->status=CQ_SYNCED;
    else v->status=CQ_CACHED;
}

void cq_reset_text(char *text,size_t size,uint64_t now,uint32_t reset,bool weekly) {
    if(reset==0){snprintf(text,size,"--");return;}
    if(reset<=now){snprintf(text,size,"0");return;}
    uint64_t remaining=(uint64_t)reset-now;
    if(weekly)snprintf(text,size,"%lud%02luh",(unsigned long)(remaining/86400),(unsigned long)(remaining%86400/3600));
    else {uint64_t minutes=(remaining+59)/60;snprintf(text,size,"%luh%02lum",(unsigned long)(minutes/60),(unsigned long)(minutes%60));}
}

bool cq_dual_layout(const cq_view_t *v) { return v->five!=CQ_NOT_APPLICABLE&&v->weekly!=CQ_NOT_APPLICABLE; }
uint8_t cq_active_metric(const cq_view_t *v,uint8_t requested) {
    if(v->five==CQ_NOT_APPLICABLE)return 1;
    if(v->weekly==CQ_NOT_APPLICABLE)return 0;
    return requested?1:0;
}
uint32_t cq_quota_color(uint8_t value) {
    return value>100?0xB3C0C5:value<20?0xFFB4AB:value<50?0xFFC968:0x72D6A6;
}
cq_layout_t cq_next_layout(cq_layout_t layout) {
    return layout==CQ_LAYOUT_FOCUS?CQ_LAYOUT_ROWS:CQ_LAYOUT_FOCUS;
}
