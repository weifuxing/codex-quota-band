#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "quota_state.h"
static void put32(uint8_t *p,uint32_t v) { for(int i=0;i<4;i++)p[i]=(uint8_t)(v>>(8*i)); }
int main(void) {
    uint8_t p[20]={2,1,68,42}; put32(p+4,1000);put32(p+8,970);put32(p+12,1200);put32(p+16,2000);
    cq_state_t s={0}; cq_view_t v;
    assert(cq_receive(&s,p,20,10)); cq_view(&s,10,&v); assert(v.status==CQ_SYNCED&&v.five==68);
    put32(p+4,1090);assert(cq_receive(&s,p,20,100));cq_view(&s,100,&v);assert(v.status==CQ_SYNCED);cq_view(&s,101,&v);assert(v.status==CQ_CACHED&&v.age==121);
    cq_disconnect(&s);cq_view(&s,102,&v);assert(v.status==CQ_OFFLINE&&v.five==68);
    p[1]=2;put32(p+4,1190);assert(cq_receive(&s,p,20,200));cq_view(&s,200,&v);assert(v.status==CQ_CACHED);
    p[1]=2;put32(p+4,1201);assert(cq_receive(&s,p,20,201));cq_view(&s,201,&v);assert(v.status==CQ_CACHED&&v.five==255&&v.weekly==42);
    cq_state_t before=s;p[0]=3;assert(!cq_receive(&s,p,20,202));assert(memcmp(&before,&s,sizeof(s))==0);
    p[0]=2;assert(!cq_receive(&s,p,19,202));p[2]=101;assert(!cq_receive(&s,p,20,202));p[2]=68;
    put32(p+8,1300);assert(!cq_receive(&s,p,20,202));put32(p+8,970);put32(p+4,1200);assert(!cq_receive(&s,p,20,202));
    put32(p+4,1300);assert(cq_receive(&s,p,20,203));cq_view(&s,249,&v);assert(v.status==CQ_OFFLINE);
    cq_state_t empty={0};cq_view(&empty,0,&v);assert(v.status==CQ_NO_DATA&&v.five==255);
    p[1]=0;p[2]=p[3]=255;put32(p+8,0);assert(cq_receive(&empty,p,20,0));cq_view(&empty,0,&v);assert(v.status==CQ_NO_DATA);
    p[1]=1;p[2]=254;p[3]=85;put32(p+4,1400);put32(p+8,1390);put32(p+12,0);put32(p+16,2000);
    assert(cq_receive(&empty,p,20,10));cq_view(&empty,10,&v);assert(v.five==254&&v.weekly==85);
    cq_disconnect(&empty);cq_view(&empty,20,&v);assert(v.status==CQ_OFFLINE&&v.five==254);
    p[1]=2;p[2]=255;put32(p+4,1410);assert(cq_receive(&empty,p,20,20));cq_view(&empty,20,&v);assert(v.five==254);
    char text[24];cq_reset_text(text,sizeof(text),1000,1000+7500,false);assert(strcmp(text,"2h05m")==0);
    cq_reset_text(text,sizeof(text),1000,1000+3*86400+8*3600,true);assert(strcmp(text,"3d08h")==0);
    assert(!cq_dual_layout(&v));assert(cq_active_metric(&v,0)==1);
    v.five=68;assert(cq_dual_layout(&v));assert(cq_active_metric(&v,0)==0);assert(cq_active_metric(&v,1)==1);
    assert(cq_quota_color(19)==0xFFB4AB);assert(cq_quota_color(20)==0xFFC968);
    assert(cq_quota_color(49)==0xFFC968);assert(cq_quota_color(50)==0x72D6A6);
    assert(cq_quota_color(CQ_UNKNOWN)==0xB3C0C5);
    assert(cq_next_layout(CQ_LAYOUT_FOCUS)==CQ_LAYOUT_ROWS);
    assert(cq_next_layout(cq_next_layout(CQ_LAYOUT_FOCUS))==CQ_LAYOUT_FOCUS);
    assert(cq_next_layout((cq_layout_t)99)==CQ_LAYOUT_FOCUS);
    cq_screen_t screen={0};
    assert(cq_screen_update(&screen,0,false,false,false));
    assert(cq_screen_update(&screen,179,false,false,false));
    assert(!cq_screen_update(&screen,180,false,false,false));
    assert(cq_screen_update(&screen,181,false,false,true));
    assert(cq_screen_update(&screen,360,false,false,false));
    assert(!cq_screen_update(&screen,361,false,false,false));
    assert(cq_screen_update(&screen,362,true,false,false));
    assert(cq_screen_update(&screen,10000,true,false,false));
    assert(cq_screen_update(&screen,10001,false,false,false));
    assert(cq_screen_update(&screen,10100,true,false,false));
    assert(cq_screen_update(&screen,10101,false,false,false));
    assert(cq_screen_update(&screen,10280,false,false,false));
    assert(!cq_screen_update(&screen,10281,false,false,false));
    assert(cq_screen_update(&screen,10282,false,true,false));
    assert(cq_screen_update(&screen,11000,false,true,false));
    int8_t dx,dy;bool positions[5][5]={0};
    for(uint64_t i=0;i<9;i++) {
        cq_screen_shift(i*60,&dx,&dy);assert(dx>=-2&&dx<=2&&dy>=-2&&dy<=2);
        assert(!positions[dx+2][dy+2]);positions[dx+2][dy+2]=true;
        int8_t x2,y2;cq_screen_shift(i*60+59,&x2,&y2);assert(dx==x2&&dy==y2);
    }
    cq_screen_shift(540,&dx,&dy);assert(dx==0&&dy==0);
    puts("quota_state: all checks passed");return 0;
}
