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
#include "b2DebugDraw.h"
#include "b2Separator.h"
#include "PhysicsSprite.h"
#include "QueryCallback.h"

#define PTM_RATIO 32

class CCBox2DLayer : public cocos2d::CCLayer {
public:
    CCBox2DLayer();
    ~CCBox2DLayer();
    
    b2World* getWorld();
	PhysicsSprite* getPhysicsSpriteAtXY(const cocos2d::CCPoint);

protected:
    b2World* initWorld();
    virtual void update(float dt);
    virtual void draw();
    
    b2World* m_pWorld;
    b2DebugDraw* m_pDebugDraw;
	map<int, PhysicsSprite*> m_sprites;

};


#endif
