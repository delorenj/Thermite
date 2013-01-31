//
//  BuildingBlock.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#ifndef Thermite_BuildingBlock_h
#define Thermite_BuildingBlock_h

#include "cocos2d.h"
#include "Box2D.h"
#include "PhysicsSprite.h"
#include "CCBox2DLayer.h"
#include "Bomb.h"

class Bomb;

class BuildingBlock : public PhysicsSprite {
public:
    BuildingBlock(CCBox2DLayer*, float, float, float);
    ~BuildingBlock();
    bool isTouchingBlock(cocos2d::CCPoint p);
    void applyBomb(cocos2d::CCPoint p, Bomb* bomb);
    CCBox2DLayer* getContext();
    
private:
    CCBox2DLayer* m_pCtx;
    cocos2d::CCTexture2D* m_pSpriteTexture; // weak ref
    list<b2Body*> m_subdivisions;
};
#endif
