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
#include "Box2d.h"
#include "PhysicsSprite.h"
#include "CCBox2DLayer.h"

class BuildingBlock : public PhysicsSprite {
public:
    BuildingBlock(CCBox2DLayer*, float, float, float);
    ~BuildingBlock();
    bool init(b2World*);
    bool isTouchingBlock(cocos2d::CCPoint p);
    
private:
    cocos2d::CCLayer* m_pCtx;
    cocos2d::CCTexture2D* m_pSpriteTexture; // weak ref
};
#endif
