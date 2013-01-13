//
//  GLES-Render.h
//  Thermite
//
//  Created by Jarad Delorenzo on 12/19/12.
//
//

#ifndef __Thermite__GLES_Render__
#define __Thermite__GLES_Render__

#include "Box2D.h"

class GLESDebugDraw : public b2Draw
{
public:
    void DrawPolygon(const b2Vec2* vertices, int32 vertexCount, const b2Color& color);
    void DrawSolidPolygon(const b2Vec2* vertices, int32 vertexCount, const b2Color& color);
    void DrawCircle(const b2Vec2& center, float32 radius, const b2Color& color);
    void DrawSolidCircle(const b2Vec2& center, float32 radius, const b2Vec2& axis, const b2Color& color);
    void DrawSegment(const b2Vec2& p1, const b2Vec2& p2, const b2Color& color);
    void DrawTransform(const b2Transform& xf);
	void DrawPoint(const b2Vec2& p, float32 size, const b2Color& color);
	void DrawString(int x, int y, const char *string, ...);
	void DrawAABB(b2AABB* aabb, const b2Color& c);
};
#endif /* defined(__Thermite__GLES_DEBUG_DRAW__) */
