//
//  b2Separator.cpp
//  Thermite
//
//  Created by Jarad Delorenzo on 1/7/13.
//
//

#include "b2Separator.h"

void b2Separator::Separate(b2Body* pBody, b2FixtureDef* pFixtureDef, vector<b2Vec2>* pVerticesVec, int scale=30) {
    int i, n=pVerticesVec->size(), j, m;
    vector<b2Vec2>* pVec = new vector<b2Vec2>();
    vector<vector<b2Vec2>* > figsVec;
    b2PolygonShape polyShape;
    
    for (i=0; i<n; i++) {
        pVec->push_back(b2Vec2(pVerticesVec->at(i).x*scale,pVerticesVec->at(i).y*scale));
    }
    
    calcShapes(pVec, figsVec);
    n = figsVec.size();
    
    for (i=0; i<n; i++) {
        pVerticesVec->clear();
        pVec = figsVec[i];
        m = pVec->size();
        for (j=0; j<m; j++) {
            pVerticesVec->push_back(b2Vec2(pVec->at(j).x/scale,pVec->at(j).y/scale));
        }

        polyShape.Set((b2Vec2*)&pVerticesVec[0], pVerticesVec->size());
        pFixtureDef->shape=&polyShape;
        pBody->CreateFixture(pFixtureDef);
    }
}
        
int b2Separator::Validate(const vector<b2Vec2>* pVerticesVec) {
    int i, n=pVerticesVec->size(), j, j2, i2, i3, d, ret=0;
    bool fl, fl2=false;
    
    for (i=0; i<n; i++) {
        i2=(i<n-1)?i+1:0;
        i3=(i>0)?i-1:n-1;
        
        fl=false;
        for (j=0; j<n; j++) {
            if (((j!=i)&&j!=i2)) {
                if (! fl) {
                    d=det(pVerticesVec->at(i).x,pVerticesVec->at(i).y,pVerticesVec->at(i2).x,pVerticesVec->at(i2).y,pVerticesVec->at(j).x,pVerticesVec->at(j).y);
                    if ((d>0)) {
                        fl=true;
                    }
                }
                
                if ((j!=i3)) {
                    j2=(j<n-1)?j+1:0;
                    if (hitSegment(pVerticesVec->at(i).x,pVerticesVec->at(i).y,pVerticesVec->at(i2).x,pVerticesVec->at(i2).y,pVerticesVec->at(j).x,pVerticesVec->at(j).y,pVerticesVec->at(j2).x,pVerticesVec->at(j2).y) != NULL) {
                        ret=1;
                    }
                }
            }
        }
        
        if (! fl) {
            fl2=true;
        }
    }
    
    if (fl2) {
        if (ret==1) {
            ret=3;
        }
        else {
            ret=2;
        }
        
    }
    return ret;
}
        
		private function calcShapes(verticesVec:Vector.<b2Vec2>):Array {
			var vec:Vector.<b2Vec2>;
			var i:int,n:int,j:int;
			var d:Number,t:Number,dx:Number,dy:Number,minLen:Number;
			var i1:int,i2:int,i3:int,p1:b2Vec2,p2:b2Vec2,p3:b2Vec2;
			var j1:int,j2:int,v1:b2Vec2,v2:b2Vec2,k:int,h:int;
			var vec1:Vector.<b2Vec2>,vec2:Vector.<b2Vec2>;
			var v:b2Vec2,hitV:b2Vec2;
			var isConvex:Boolean;
			var figsVec:Array=[],queue:Array=[];
            
			queue.push(verticesVec);
            
			while (queue.length) {
				vec=queue[0];
				n=vec.length;
				isConvex=true;
                
				for (i=0; i<n; i++) {
					i1=i;
					i2=(i<n-1)?i+1:i+1-n;
					i3=(i<n-2)?i+2:i+2-n;
                    
					p1=vec[i1];
					p2=vec[i2];
					p3=vec[i3];
                    
					d=det(p1.x,p1.y,p2.x,p2.y,p3.x,p3.y);
					if ((d<0)) {
						isConvex=false;
						minLen=Number.MAX_VALUE;
                        
						for (j=0; j<n; j++) {
							if (((j!=i1)&&j!=i2)) {
								j1=j;
								j2=(j<n-1)?j+1:0;
                                
								v1=vec[j1];
								v2=vec[j2];
                                
								v=hitRay(p1.x,p1.y,p2.x,p2.y,v1.x,v1.y,v2.x,v2.y);
                                
								if (v) {
									dx=p2.x-v.x;
									dy=p2.y-v.y;
									t=dx*dx+dy*dy;
                                    
									if ((t<minLen)) {
										h=j1;
										k=j2;
										hitV=v;
										minLen=t;
									}
								}
							}
						}
                        
						if ((minLen==Number.MAX_VALUE)) {
							err();
						}
                        
						vec1=new Vector.<b2Vec2>  ;
						vec2=new Vector.<b2Vec2>  ;
                        
						j1=h;
						j2=k;
						v1=vec[j1];
						v2=vec[j2];
                        
						if (! pointsMatch(hitV.x,hitV.y,v2.x,v2.y)) {
							vec1.push(hitV);
						}
						if (! pointsMatch(hitV.x,hitV.y,v1.x,v1.y)) {
							vec2.push(hitV);
						}
                        
						h=-1;
						k=i1;
						while (true) {
							if ((k!=j2)) {
								vec1.push(vec[k]);
							}
							else {
								if (((h<0)||h>=n)) {
									err();
								}
								if (! this.isOnSegment(v2.x,v2.y,vec[h].x,vec[h].y,p1.x,p1.y)) {
									vec1.push(vec[k]);
								}
								break;
							}
                            
							h=k;
							if (((k-1)<0)) {
								k=n-1;
							}
							else {
								k--;
							}
						}
                        
						vec1=vec1.reverse();
                        
						h=-1;
						k=i2;
						while (true) {
							if ((k!=j1)) {
								vec2.push(vec[k]);
							}
							else {
								if (((h<0)||h>=n)) {
									err();
								}
								if (((k==j1)&&! this.isOnSegment(v1.x,v1.y,vec[h].x,vec[h].y,p2.x,p2.y))) {
									vec2.push(vec[k]);
								}
								break;
							}
                            
							h=k;
							if (((k+1)>n-1)) {
								k=0;
							}
							else {
								k++;
							}
						}
                        
						queue.push(vec1,vec2);
						queue.shift();
                        
						break;
					}
				}
                
				if (isConvex) {
					figsVec.push(queue.shift());
				}
			}
            
			return figsVec;
		}
        
		private function hitRay(x1:Number,y1:Number,x2:Number,y2:Number,x3:Number,y3:Number,x4:Number,y4:Number):b2Vec2 {
			var t1:Number=x3-x1,t2:Number=y3-y1,t3:Number=x2-x1,t4:Number=y2-y1,t5:Number=x4-x3,t6:Number=y4-y3,t7:Number=t4*t5-t3*t6,a:Number;
            
			a=(((t5*t2)-t6*t1)/t7);
			var px:Number=x1+a*t3,py:Number=y1+a*t4;
			var b1:Boolean=isOnSegment(x2,y2,x1,y1,px,py);
			var b2:Boolean=isOnSegment(px,py,x3,y3,x4,y4);
            
			if ((b1&&b2)) {
				return new b2Vec2(px,py);
			}
            
			return null;
		}
        
		private function hitSegment(x1:Number,y1:Number,x2:Number,y2:Number,x3:Number,y3:Number,x4:Number,y4:Number):b2Vec2 {
			var t1:Number=x3-x1,t2:Number=y3-y1,t3:Number=x2-x1,t4:Number=y2-y1,t5:Number=x4-x3,t6:Number=y4-y3,t7:Number=t4*t5-t3*t6,a:Number;
            
			a=(((t5*t2)-t6*t1)/t7);
			var px:Number=x1+a*t3,py:Number=y1+a*t4;
			var b1:Boolean=isOnSegment(px,py,x1,y1,x2,y2);
			var b2:Boolean=isOnSegment(px,py,x3,y3,x4,y4);
            
			if ((b1&&b2)) {
				return new b2Vec2(px,py);
			}
            
			return null;
		}
        
		private function isOnSegment(px:Number,py:Number,x1:Number,y1:Number,x2:Number,y2:Number):Boolean {
			var b1:Boolean=((((x1+0.1)>=px)&&px>=x2-0.1)||(((x1-0.1)<=px)&&px<=x2+0.1));
			var b2:Boolean=((((y1+0.1)>=py)&&py>=y2-0.1)||(((y1-0.1)<=py)&&py<=y2+0.1));
			return ((b1&&b2)&&isOnLine(px,py,x1,y1,x2,y2));
		}
        
		private function pointsMatch(x1:Number,y1:Number,x2:Number,y2:Number):Boolean {
			var dx:Number=(x2>=x1)?x2-x1:x1-x2,dy:Number=(y2>=y1)?y2-y1:y1-y2;
			return ((dx<0.1)&&dy<0.1);
		}
        
		private function isOnLine(px:Number,py:Number,x1:Number,y1:Number,x2:Number,y2:Number):Boolean {
			if ((((x2-x1)>0.1)||x1-x2>0.1)) {
				var a:Number=(y2-y1)/(x2-x1),possibleY:Number=a*(px-x1)+y1,diff:Number=(possibleY>py)?possibleY-py:py-possibleY;
				return (diff<0.1);
			}
            
			return (((px-x1)<0.1)||x1-px<0.1);
		}
        
		private function det(x1:Number,y1:Number,x2:Number,y2:Number,x3:Number,y3:Number):Number {
			return x1*y2+x2*y3+x3*y1-y1*x2-y2*x3-y3*x1;
		}
        
		private function err():void {
			throw new Error("A problem has occurred. Use the Validate() method to see where the problem is.");
		}
	}
}