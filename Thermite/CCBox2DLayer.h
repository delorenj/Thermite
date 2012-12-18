//
//  CCBox2DLayer.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/14/12.
//
//

#ifndef Thermite_CCBox2DLayer_h
#define Thermite_CCBox2DLayer_h

#include "cocos2d.h"
#include "Box2D.h"

#define PTM_RATIO 32

class CCBox2DLayer : public cocos2d::CCLayer {
public:
    CCBox2DLayer();
    ~CCBox2DLayer();
    
    virtual b2World* getWorld();
    
private:
    b2World* initWorld();
    virtual void update(float dt);
    virtual void draw();
    
    b2World* m_pWorld;
};


#endif
