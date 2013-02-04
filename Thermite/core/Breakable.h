#ifndef Thermite_Breakable_h
#define Thermite_Breakable_h


#include "cocos2d.h"
#include "Box2D.h"
#include "PhysicsSprite.h"
#include "CCBox2DLayer.h"
#include "Bomb.h"
#include "b2Separator.h"
#include "NonConvexHull.h"

class Bomb;

class Breakable {
public:
    Breakable(CCBox2DLayer* ctx, float w, float h, float x, float y, bool structure=true);
	Breakable(CCBox2DLayer* ctx, vector<b2Vec2>& shape, float x, float y, bool structure=true);
    ~Breakable();
    bool isTouching(const cocos2d::CCPoint p) const;
	bool isStructure() const;
	void setStructure(const bool val);
    void applyBomb(Bomb& bomb);
    CCBox2DLayer* getContext() const;
	PhysicsSprite* getPhysicsSprite() { return m_pPhysicsSprite; }
    
private:
	void setBodyUserData() { m_pPhysicsSprite->getPhysicsBody()->SetUserData(this); }
	PhysicsSprite* m_pPhysicsSprite;
	cocos2d::CCTexture2D* m_pSpriteTexture;
    CCBox2DLayer* m_pCtx;
	NonConvexHull* m_pHull;
};
#endif
