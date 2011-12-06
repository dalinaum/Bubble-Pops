#pragma version(1)
#pragma rs java_package_name(com.bubble.pops)

#include "balls.rsh"

static const int SCORE_EVENT = 1;
static const int POP_EVENT = 3;

float2 gMinPos = {0.f, 0.f};
float2 gMaxPos = {1280.f, 700.f};

static float2 touchPos[10];
static int touchState[10];
static float deltaHistory[10][10][2];
static int deltaHistoryIndex[10];

int scores[2];

static float2 compAvgVel(float deltaHistoryVect[10][2]) {
	float2 avgs = {0.0f, 0.0f};
	for (int i = 0; i < 10; i ++) {
		avgs.x += deltaHistoryVect[i][0];
		avgs.y += deltaHistoryVect[i][1];
	}
	avgs.x = avgs.x*1.0;
	avgs.y = avgs.y*1.0;
	if (avgs.x > 250.f) { avgs.x = 250.f; }
	if (avgs.x < -250.f) { avgs.x = -250.f; }
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

void root(const Ball_t *ballIn, Ball_t *ballOut, const void *vctl, uint32_t x) {
    BallControl_t *ctl = (BallControl_t *) vctl;
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
    ballOut->position = ballIn->position;
	if (ballIn->pointerId > -1 && touchState[ballIn->pointerId] == 2) {
	
		// Explode the ball if it is dragged over the half way line
		if ((ballOut->team && touchPos[ballIn->pointerId].x > 740) ||
		(!ballOut->team && touchPos[ballIn->pointerId].x < 540)) {
			
			if (ballOut->active) {
				ballOut->active = 0;
	        	rsSendToClient(POP_EVENT);
	        }
			
		// Otherwise update the tracking history
		} else if (fabs(touchPos[ballIn->pointerId].x - ballIn->position.x) > 0.f){ 
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
	        if (bPtr[xin].active && sqrt(len2) < 5.5f * ballIn->size && vec.x != 0 && vec.y != 0) {
	            //float minDist = ballIn->size + bPtr[xin].size;
	            float forceScale = ballIn->size * bPtr[xin].size;
	            forceScale *= forceScale;
	
	            // Collision
				rsDebug("Collision detected", 0);	            
	            float2 axis = normalize(vec);
	            float e1 = dot(axis, ballIn->delta);
	            float e2 = dot(axis, bPtr[xin].delta);
	            //if (e1 < 0.5f || e2 < 0.5f) { e1 = 0.f; e2 = 0.f; }
	            float e = (e1 - e2);
	            if (e1 > 0) {
	                fv -= 8.f * axis * e;
	            } else {
	                fv += 8.f * axis * e;
	            }
	        }
	    }
		
		fv *= ctl->dt;
	
	    for (int i=0; i < 10; i++) {
	        if (touchState[i] != 0) {
	            float2 vec = touchPos[i] - ballIn->position;
	            float2 vec2 = vec * vec;
	            float len2 = max(2.f, vec2.x + vec2.y);
	            if(len2 < 35.f*35.f){
	            	if (ballOut->active && ((ballIn->team && touchPos[i].x >= 640) ||
	            		(!ballIn->team && touchPos[i].x <= 640))) {
	            		ballOut->active = 0;
	            		rsSendToClient(POP_EVENT);
	            		if (ballIn->team == 0) {scores[1]++;}
	        			else {scores[0]++;}
	            	} else {
	            		//rsDebug("Setting id",i);
	            		ballOut->pointerId = i;
	            	}
	            }            
	        }
	    }
	
		float2 candidate = (ballIn->delta * (1.f - 0.02f)) + fv;
		
	    ballOut->delta = candidate;
	    ballOut->position = ballIn->position + (ballOut->delta * ctl->dt);	
	
	    const float wallForce = 400.f;
	    if (ballOut->position.x > (gMaxPos.x - 20.f)) {
	        if (ballIn->team && ballOut->active) {
	        	scores[ballIn->team]+=3;
	        	ballOut->active = 0;
//	        	rsDebug("SCORE! ",ballIn->team);
//	        	rsDebug("With goals: ",scores[ballIn->team]);
				rsSendToClient(SCORE_EVENT);
	        }
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
	    	if (!ballIn->team && ballOut->active) {
	    		scores[ballIn->team]+=3;
	        	ballOut->active = 0;
	        	rsSendToClient(SCORE_EVENT);
//	        	rsDebug("SCORE! ",ballIn->team);
//	        	rsDebug("With goals: ",scores[ballIn->team]);
	        }
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
	    
	    //Update rendered scores
	    ctl->scores[0] = scores[0];
	    ctl->scores[1] = scores[1];
	}
	
    ballOut->size = ballIn->size;

    //rsDebug("physics pos out", ballOut->position);
}

