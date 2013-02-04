#include "Prototype.h"

using namespace cocos2d;

Prototype::Prototype() {
	setTouchEnabled(true);
	CCSize size = CCDirector::sharedDirector()->getWinSize();
	srand ( time(NULL) );
	m_bodyDef.type = b2_dynamicBody;
    m_fixtureDef.restitution = 0.4f;
    m_fixtureDef.friction = 0.2f;
    m_fixtureDef.density = 4;
    m_centerPoint = CCPointMake(0.5*size.width, 0.5*size.height);

	try {
		testSimple();
		testSeparator();
	}
	catch(exception e) {
		CCLog("Oops...%s", e.what());
	}

	scheduleUpdate();

}


Prototype::~Prototype() {

}

CCScene* Prototype::scene() {
    CCScene* scene = CCScene::create();
    CCLayer* layer = new Prototype();
    scene->addChild(layer,0);
    layer->release();
    return scene;
}

void Prototype::testSimple() {
	Breakable* pStruct = new Breakable(static_cast<CCBox2DLayer*>(this), 256, 256, m_centerPoint.x, m_centerPoint.y, true);
}

void Prototype::testSeparator() {

    vector<b2Vec2>* vec = new vector<b2Vec2>();
    vec->push_back(b2Vec2(-4, -4));
    vec->push_back(b2Vec2(4, -4));
    vec->push_back(b2Vec2(4, 0));
    vec->push_back(b2Vec2(0, 0));
    vec->push_back(b2Vec2(0, 4));
	vec->push_back(b2Vec2(-4, 4));

	Breakable* pStruct = new Breakable(static_cast<CCBox2DLayer*>(this), *vec, m_centerPoint.x-150, m_centerPoint.y+300, false);

}

void Prototype::testPlaceBomb(b2Body* body, const CCPoint touchPoint, const float radius) {
	CCLog("Breaking Body: %d", static_cast<PhysicsSprite*>(body->GetUserData())->getTag());

	b2Separator sep;
	vector<b2Vec2>* bombShape;
	vector<b2Vec2> breakableShape; //should store these in the breakable object since it will be filled with unneeded fixtures. Need the hull
	vector<b2Vec2>* newStructure = new vector<b2Vec2>();
	vector<b2Vec2>* brokenStructure = new vector<b2Vec2>();
	b2FixtureDef bombFixture = m_fixtureDef;
	b2Vec2 worldPoint =  b2Vec2(touchPoint.x/PTM_RATIO, touchPoint.y/PTM_RATIO);
	b2Vec2 localPoint = body->GetLocalPoint(worldPoint);
	bool valid = true;
	bombFixture.isSensor = true;

	breakableShape.push_back(b2Vec2(-4, 4));
	breakableShape.push_back(b2Vec2(-4, -4));
	breakableShape.push_back(b2Vec2(4, -4));
	breakableShape.push_back(b2Vec2(4, 4));

	do {
		do {
			 bombShape = generateBlastShape(radius, 20, 0.5f);
		} while (sep.Validate(*bombShape) != 0);

		try{
			sep.Validate(*bombShape);
		} catch(b2SeparatorException& e) {
			CCLog("b2Separator Exception: %s", e.what());
		}

		for(int i=0; i<bombShape->size(); i++) {
			bombShape->at(i).x += localPoint.x;
			bombShape->at(i).y += localPoint.y;
		}

		try {
			sep.Separate(body, &bombFixture, bombShape, PTM_RATIO);
			valid = true;
		} catch(b2SeparatorException& e) {
			valid = false;
			CCLog("b2Separator Exception: %s", e.what());
			continue;
		}
	} while(!valid);


	int i=0; //vertex index
	b2Vec2* lastVertex = NULL;
	bool lastState = false;
	bool specialWindingNeeded = false;
	bool specialWindingStarted = false;
	int crossoverCount = 0;
	deque<b2Vec2> specialWindingStack;
	for(vector<b2Vec2>::iterator it = bombShape->begin(); it != bombShape->end(); it++) {
		b2Vec2 v = *it;
		int j=0; //fixture index

		CCLog("Blast Vertex %d of %d:", i++, bombShape->size());

		for(b2Fixture* fix = body->GetFixtureList(); fix; fix=fix->GetNext()) {
			//Iterate over non-bomb fixtures
			b2Shape::Type shapeType = fix->GetType();
			if(shapeType != b2Shape::e_polygon) {
				CCLog("Non-polygon encountered. Continuing...");
				continue;
			}
			if(!fix->IsSensor()) {
				CCLog("Testing breakable fixture: %d", j);
				b2PolygonShape* shape = (b2PolygonShape*)fix->GetShape();
				//At this point, shape should be a blow-uppable fixture
				//Test if bomb vertex is in fixture 					
				b2Transform transform; 
				transform.SetIdentity();
				bool pointIn = shape->TestPoint(transform, v);

				// Handle crossovers
				if(it != bombShape->begin()) {
					if(lastState != pointIn) {
						CCLog("Crossover Detected at (%f, %f)", v.x, v.y);
						crossoverCount++;
						b2Vec2 crossoverVertex;
						//RayCasting only works with world coordinates
						b2Vec2 p1 = body->GetWorldPoint(*lastVertex);
						b2Vec2 p2 = body->GetWorldPoint(v);
						try {
							//Make sure RayCast is coming from the outside of the fixture or it will not intersect
							if(pointIn) {
								crossoverVertex = getCrossoverVertex(*fix, p1, p2);
								//Convert world coordinates back into local coordinates.
								crossoverVertex = body->GetLocalPoint(crossoverVertex);
								if(specialWindingNeeded) {
									specialWindingStarted = true;
									specialWindingStack.push_front(crossoverVertex);
									specialWindingStack.push_front(v);
								} else {
									newStructure->push_back(crossoverVertex);
									newStructure->push_back(v);
								}
							} else  {
								crossoverVertex = getCrossoverVertex(*fix, p2, p1);
								//Convert world coordinates back into local coordinates.
								crossoverVertex = body->GetLocalPoint(crossoverVertex);
								newStructure->push_back(crossoverVertex);
								brokenStructure->push_back(crossoverVertex);
							}

							
						}catch(exception e) {
							// wtf?
						}
					} else { // if last state and current state are same
						if(pointIn) {
							if(specialWindingStarted) {
								specialWindingStack.push_front(v);
							} else {
								newStructure->push_back(v);
								brokenStructure->push_back(v);
							}
						}
					}
				// Set bomb vertices begin from INSIDE the fixture, special care must be taken
				// to wind the vertices in CCW order.
				} else {
					if(pointIn) { //on first iteration...
						specialWindingNeeded = true;
						newStructure->push_back(v);
						brokenStructure->push_back(v);
					}
				}
				lastState = pointIn;
				CCLog("\tVertex %d %s fixture %d: (%f, %f)", i+1, pointIn ? "is in" : "is not in", j++, v.x, v.y); 

			}
		}
		lastVertex = &*it;
	}

	//Done iterating over bomb. Process pending specialWindingStack and merge vertices into new/broken structures
	if(specialWindingStack.size() > 0) {
		while(!specialWindingStack.empty()) {
			b2Vec2 vert = specialWindingStack.at(0);
			newStructure->push_back(vert);
			brokenStructure->push_back(vert);
			specialWindingStack.pop_front();
		}
	}

	newStructure->push_back(breakableShape[0]);
	newStructure->push_back(breakableShape[1]);
	newStructure->push_back(breakableShape[2]);
	brokenStructure->push_back(breakableShape[3]);

	m_pWorld->DestroyBody(body);
	b2Body* newBreakable = m_pWorld->CreateBody(&m_bodyDef);
	try{
		sep.Validate(*newStructure);
	} catch(b2SeparatorException& e) {
		CCLog("b2Separator Exception: %s", e.what());
	}

	try {
		sep.Separate(newBreakable, &m_fixtureDef, newStructure, PTM_RATIO);
	} catch(b2SeparatorException& e) {
		CCLog("b2Separator Exception: %s", e.what());
	}
}

b2Vec2 Prototype::getCrossoverVertex(const b2Fixture& fixture, const b2Vec2& p1, const b2Vec2& p2) {
    b2RayCastInput input;
    input.p1 = p1;
    input.p2 = p2;
    input.maxFraction = 1;
    float closestFraction = 1;
    bool intersected = false;
    b2RayCastOutput output;

    if (!fixture.RayCast(&output, input, 0)) {
       CCLog("No intersection found...This should not have happened.");
	   throw exception();
	}

    if (closestFraction > output.fraction)
        closestFraction = output.fraction; 

    b2Vec2 hitPoint = input.p1 + closestFraction * (input.p2 - input.p1);
    return hitPoint;
}

vector<b2Vec2>* Prototype::generateBlastShape(float radius, int segments, float roughness) {
    vector<b2Vec2>* vec = new vector<b2Vec2>();
	float delta = 2.0f*b2_pi / segments;
	float radius_threshold = radius * roughness;
	float theta = 0;
	for(int i=0; i<segments; i++, theta+=delta) {
		float x,y,r;
		r = radius + CCRANDOM_MINUS1_1()*radius_threshold;
		x = r*cos(theta);
		y = r*sin(theta);
		vec->push_back(b2Vec2(x, y));
	}
	return vec;
}

CCPoint Prototype::touchToPoint(CCTouch* pTouch) {
    return CCDirector::sharedDirector()->convertToGL(pTouch->getLocationInView());
}

void Prototype::ccTouchesBegan(CCSet *pTouches, CCEvent *pEvent) {
	PhysicsSprite* sprite;

    for (CCSetIterator it = pTouches->begin(); it != pTouches->end(); it++) {
        CCTouch* touch = dynamic_cast<CCTouch*>(*it);
		CCPoint touchPoint = touchToPoint(touch);
		sprite = getPhysicsSpriteAtXY(touchPoint);

		if(sprite != NULL) {
//			testPlaceBomb(sprite->getPhysicsBody(), touchPoint, 1.75f );
		}

    }
}

void Prototype::ccTouchesEnded(CCSet *pTouches, CCEvent *pEvent) {
    
}

void Prototype::ccTouchesMoved(CCSet *pTouches, CCEvent *pEvent) {
    
}
