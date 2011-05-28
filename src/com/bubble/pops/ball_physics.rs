#pragma version(1)
#pragma rs java_package_name(com.bubble.pops)

#include "balls.rsh"

float2 gMinPos = {0.f, 0.f};
float2 gMaxPos = {1280.f, 700.f};

static float2 touchPos[10];
static int touchState[10];
static float deltaHistory[10][10][2];
static int deltaHistoryIndex[10];

static float2 compAvgVel(float deltaHistoryVect[10][2]) {
	float2 avgs = {0.0f, 0.0f};
	for (int i = 0; i < 10; i ++) {
		avgs.x += deltaHistoryVect[i][0];
		avgs.y += deltaHistoryVect[i][1];
	}
	avgs.x = avgs.x*3;
	avgs.y = avgs.y*3;
	return avgs;
}

void touch(float x, float y, float pressure, int id) {
    if (id >= 10) {
        return;
    }

    touchPos[id].x = x;
    touchPos[id].y = y;
    
    if(pressure > 0){
    	if(touchState[id] == 0){
    		touchState[id] = 1;
    	}else{
    		touchState[id] = 2;
    	}
    }else{
    	touchState[id] = 0;
    }
}

void root(const Ball_t *ballIn, Ball_t *ballOut, const BallControl_t *ctl, uint32_t x) {
    float2 fv = {0, 0};
    float2 pos = ballIn->position;

    int arcID = -1;
    float arcInvStr = 100000;
    //rsDebug("ballIn->pointerId", ballIn->pointerId);
    //rsDebug("touchPos[ballIn->pointerId]", touchPos[ballIn->pointerId]);
    ballOut->pointerId = ballIn->pointerId;
    ballOut->size = ballIn->size;
    ballOut->team = ballIn->team;
    ballOut->active = ballIn->active;
	if (ballIn->pointerId > -1 && touchState[ballIn->pointerId] == 2) {
		if (fabs(touchPos[ballIn->pointerId].x - ballIn->position.x) > 0.f){ 
		//fabs(touchPos[ballIn->pointerId].y - ballIn->position.y) > 30.f) {
			ballOut->delta = touchPos[ballIn->pointerId] - ballIn->position;
			deltaHistory[ballIn->pointerId][deltaHistoryIndex[ballIn->pointerId]][0] = ballOut->delta.x;
			deltaHistory[ballIn->pointerId][deltaHistoryIndex[ballIn->pointerId]][1] = ballOut->delta.y;
			deltaHistoryIndex[ballIn->pointerId] = (deltaHistoryIndex[ballIn->pointerId] + 1) % 10;
			ballOut->delta = compAvgVel(deltaHistory[ballIn->pointerId]);
			rsDebug("ballOut->delta", ballOut->delta);
		}
		ballOut->position = touchPos[ballIn->pointerId];
	} else if(touchState[ballIn->pointerId] == 0){
		ballOut->pointerId = -1;
	    const Ball_t * bPtr = rsGetElementAt(ctl->ain, 0);
	    for (uint32_t xin = 0; xin < ctl->dimX; xin++) {
	        float2 vec = bPtr[xin].position - pos;
	        float2 vec2 = vec * vec;
	        float len2 = vec2.x + vec2.y;
	
	        if (len2 < 10000) {
	            //float minDist = ballIn->size + bPtr[xin].size;
	            float forceScale = ballIn->size * bPtr[xin].size;
	            forceScale *= forceScale;
	
	            // Collision
	            /*float2 axis = normalize(vec);
	            float e1 = dot(axis, ballIn->delta);
	            float e2 = dot(axis, bPtr[xin].delta);
	            float e = (e1 - e2) * 0.0f//0.45f;
	            if (e1 > 0) {
	                fv -= axis * e;
	            } else {
	                fv += axis * e;
	            }*/
	        }
	    }
	
	    fv *= ctl->dt;
	
	    for (int i=0; i < 10; i++) {
	        if (touchState[i] != 0) {
	            float2 vec = touchPos[i] - ballIn->position;
	            float2 vec2 = vec * vec;
	            float len2 = max(2.f, vec2.x + vec2.y);
	            if(len2 < 30.f*30.f){
	            	//ballOut->active = 0;
	            	//rsDebug("Setting id",i);
	            	ballOut->pointerId = i;
	            }            
	        }
	    }
	
	    ballOut->delta = (ballIn->delta * (1.f - 0.02f)) + fv;
	    ballOut->position = ballIn->position + (ballOut->delta * ctl->dt);
	
	    const float wallForce = 400.f;
	    if (ballOut->position.x > (gMaxPos.x - 20.f)) {
	        float d = gMaxPos.x - ballOut->position.x;
	        if (d < 0.f) {
	            if (ballOut->delta.x > 0) {
	                ballOut->delta.x *= -0.7;
	            }
	            ballOut->position.x = gMaxPos.x;
	        } else {
	            ballOut->delta.x -= min(wallForce / (d * d), 10.f);
	        }
	    }
	
	    if (ballOut->position.x < (gMinPos.x + 20.f)) {
	        float d = ballOut->position.x - gMinPos.x;
	        if (d < 0.f) {
	            if (ballOut->delta.x < 0) {
	                ballOut->delta.x *= -0.7;
	            }
	            ballOut->position.x = gMinPos.x + 1.f;
	        } else {
	            ballOut->delta.x += min(wallForce / (d * d), 10.f);
	        }
	    }
	
	    if (ballOut->position.y > (gMaxPos.y - 20.f)) {
	        float d = gMaxPos.y - ballOut->position.y;
	        if (d < 0.f) {
	            if (ballOut->delta.y > 0) {
	                ballOut->delta.y *= -0.7;
	            }
	            ballOut->position.y = gMaxPos.y;
	        } else {
	            ballOut->delta.y -= min(wallForce / (d * d), 10.f);
	        }
	    }
	
	    if (ballOut->position.y < (gMinPos.y + 20.f)) {
	        float d = ballOut->position.y - gMinPos.y;
	        if (d < 0.f) {
	            if (ballOut->delta.y < 0) {
	                ballOut->delta.y *= -0.7;
	            }
	            ballOut->position.y = gMinPos.y + 1.f;
	        } else {
	            ballOut->delta.y += min(wallForce / (d * d * d), 10.f);
	        }
	    }
	}
	
    ballOut->size = ballIn->size;

    //rsDebug("physics pos out", ballOut->position);
}

