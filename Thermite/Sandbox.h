//
//  Sandbox.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#ifndef __THERMITE_SANDBOX_H__
#define __THERMITE_SANDBOX_H__

#include "cocos2d.h"
#include "Box2D.h"
#include "CCBox2DLayer.h"
#include "BuildingBlock.h"
#include "LegoBomb.h"
#include "b2Separator.h"

using namespace cocos2d;

class Sandbox : public CCBox2DLayer {
public:
    Sandbox();
    ~Sandbox();
    
    static cocos2d::CCScene* scene();
    
private:
    void initBlocks();
    void initB2SeparatorExample();
    CCPoint touchToPoint(CCTouch* pTouch);
    void ccTouchesBegan(CCSet* pTouches, CCEvent* pEvent);
    void ccTouchesMoved(CCSet* pTouches, CCEvent* pEvent);
    void ccTouchesEnded(CCSet* pTouches, CCEvent* pEvent);
    
    bool isTouchingBlock(CCTouch* touch);
    
    list<BuildingBlock*> m_pBuildingBlocks;
    CCLayerColor* colorLayer;
};

#endif
