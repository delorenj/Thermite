//
//  Sandbox.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 12/10/12.
//
//

#include "Sandbox.h"

using namespace cocos2d;

Sandbox::Sandbox() {
	this->setTouchEnabled(true);
	this->colorLayer = new CCLayerColor;
	this->colorLayer->initWithColor( ccc4(180, 180, 180, 255) );
//	this->addChild(this->colorLayer, 1);
    CCSize s = CCDirector::sharedDirector()->getWinSize();
        
    CCLabelTTF *label = CCLabelTTF::create("Sandbox Mode", "Marker Felt", 32);
    this->addChild(label, 2);
    label->setColor(ccc3(0,0,255));
    label->setPosition(ccp( s.width/2, s.height-50));
    
    //this->initBlocks();
    this->initB2SeparatorExample();
    
    scheduleUpdate();
}

Sandbox::~Sandbox() {
    	
}

CCScene* Sandbox::scene() {
    CCScene* scene = CCScene::create();
    CCLayer* layer = new Sandbox();
    scene->addChild(layer,0);
    layer->release();
    return scene;
}

void Sandbox::initB2SeparatorExample() {
        CCSize s = CCDirector::sharedDirector()->getWinSize();

        b2Body *body;
        b2BodyDef *bodyDef = new b2BodyDef();
        b2FixtureDef *fixtureDef = new b2FixtureDef();

        // 1) We create a b2Separator instance.
        b2Separator* sep = new b2Separator();

        // 2) Then we create a b2Body instance. This is where the fixtures of the non-polygon shape will be stored.
        bodyDef->type = b2_dynamicBody;
        bodyDef->position.Set(0.5*s.width/30, 0.5*s.height/30);
        body = getWorld()->CreateBody(bodyDef);
        
        // 3) We also need a b2FixtureDef instance, so that the new fixtures can inherit its properties.
        fixtureDef->restitution = 0.4;
        fixtureDef->friction = 0.2;
        fixtureDef->density = 4;
        
        // 4) And what is of most importance - we need a Vector of b2Vec2 instances so that we can pass the vertices! 
        // Remember, we need the vertices in clockwise order! For more information, read the documentation for the b2Separator.Separate() method.
        vector<b2Vec2>* vec = new vector<b2Vec2>();
        vec->push_back(b2Vec2(-100/30, -100/30));
        vec->push_back(b2Vec2(100/30, -100/30));
        vec->push_back(b2Vec2(100/30, 0));
        vec->push_back(b2Vec2(0, 0));
        vec->push_back(b2Vec2(-100/30, 100/30));
        
        // If you want to be sure that the vertices are entered correctly, use the b2Separator.Validate() method!
        // Refer to the documentation of b2Separate.Validate() to see what it does and the values it returns.
        if(sep->Validate(*vec)==0)
        {
            CCLog("Yay! Those vertices are good to go!");
        }
        else
        {
            CCLog("Oh, I guess you effed something up :(");
        }
        
        // 5) And finally, we pass the b2Body, b2FixtureDef and Vector.<b2Vec2> instances as parameters to the Separate() method!
        // It separates the non-convex shape into convex shapes, creates the fixtures and adds them to the body for us! Sweet, eh?
        sep->Separate(body, fixtureDef, vec, 30);
        
//        // Assigning an event listener, which allows us to call update() every frame.
//        stage.addEventListener(Event.ENTER_FRAME, update);

}

void Sandbox::initBlocks() {
    CCSize s = CCDirector::sharedDirector()->getWinSize();

    m_pBuildingBlocks.push_front(new BuildingBlock(this, 256, s.width/2, s.height/2));
    m_pBuildingBlocks.push_front(new BuildingBlock(this, 150, 100, 400));
}

CCPoint Sandbox::touchToPoint(CCTouch* pTouch) {
    return CCDirector::sharedDirector()->convertToGL(pTouch->getLocationInView());
}

void Sandbox::ccTouchesBegan(CCSet *pTouches, CCEvent *pEvent) {
    for (auto it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);

        for(auto it = m_pBuildingBlocks.begin(); it != m_pBuildingBlocks.end(); it ++) {
            BuildingBlock* b = dynamic_cast<BuildingBlock*>(*it);
            if(b->isTouchingBlock(touchToPoint(touch))) {
                LegoBomb* bomb = new LegoBomb();
                b->applyBomb(touchToPoint(touch), bomb);
            }
        }
    }
}

void Sandbox::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Sandbox::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}