//
//  PhysicsSprite.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#ifndef Thermite_PhysicsSprite_h
#define Thermite_PhysicsSprite_h

#include "cocos2d.h"
#include "Box2D.h"

//REFACTOR:  This is defined twice...which is fucking retarded.
#define PTM_RATIO 32

class PhysicsSprite : public cocos2d::CCSprite
{
public:
    PhysicsSprite();
	~PhysicsSprite();
    void setPhysicsBody(b2Body * body);
    b2Body* getPhysicsBody();
    virtual bool isDirty(void);
    virtual cocos2d::CCAffineTransform nodeToParentTransform(void);
	void SetUserData(void* data);
	void* GetUserData() { return m_pUserData; }

private:
    b2Body* m_pBody;    // strong ref
	void* m_pUserData;
};


#endif
 