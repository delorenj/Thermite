#ifndef Thermite_Breakable_h
#define Thermite_Breakable_h


#include "cocos2d.h"
#include "Box2d.h"
#include "PhysicsSprite.h"
#include "CCBox2DLayer.h"
#include "Bomb.h"
#include "b2Separator.h"

class Bomb;

class Breakable {
public:
    Breakable(CCBox2DLayer*, const char*, float, float);
    ~Breakable();
    bool isTouching(cocos2d::CCPoint p);
    void applyBomb(cocos2d::CCPoint p, Bomb* bomb);
    CCBox2DLayer* getContext();
    
private:
	PhysicsSprite* m_pPhysicsSprite;
	cocos2d::CCTexture2D* m_pSpriteTexture;
    CCBox2DLayer* m_pCtx;
};
#endif
