LOCAL_PATH 			:= $(call my-dir)
COCOS2DX_ROOT 			:= /home/delorenj/cocos2d-x
COCOS2DX_ANDROID_PREBUILT	:= $(COCOS2DX_ROOT)/platform/third_party/android/prebuilt
COCOS2DX			:= $(COCOS2DX_ROOT)/cocos2dx
BOX2D_INCLUDE			:= $(COCOS2DX_ROOT)/external/Box2D
COCOS_DENSHION_INCLUDE		:= $(COCOS2DX_ROOT)/CocosDenshion/include
THERMITE_CORE			:= $(LOCAL_PATH)/../../core
THERMITE_CORE_REL		:= ../../core

include $(CLEAR_VARS)

LOCAL_MODULE := game_shared

LOCAL_MODULE_FILENAME := libthermite

LOCAL_SRC_FILES := thermite/main.cpp \
                   $(THERMITE_CORE_REL)/Prototype.cpp \
                   $(THERMITE_CORE_REL)/AppDelegate.cpp \
                   $(THERMITE_CORE_REL)/CCBox2DLayer.cpp \
                   $(THERMITE_CORE_REL)/PhysicsSprite.cpp \
                   $(THERMITE_CORE_REL)/b2Separator.cpp \
                   $(THERMITE_CORE_REL)/b2DebugDraw.cpp

LOCAL_C_INCLUDES := 	$(THERMITE_CORE) \
		    	$(BOX2D_INCLUDE) \
			$(BOX2D_INCLUDE)/../ \
			$(COCOS_DENSHION_INCLUDE) \
			$(COCOS2DX) \
			$(COCOS2DX)/include \
			$(COCOS2DX)/platform \
			$(COCOS2DX)/platform/android \
			$(COCOS2DX)/kazmath/include
		
LOCAL_WHOLE_STATIC_LIBRARIES := cocos2dx_static cocosdenshion_static cocos_extension_static box2d_static
            
include $(BUILD_SHARED_LIBRARY)

$(call import-module,CocosDenshion/android) \
$(call import-module,cocos2dx) \
$(call import-module,extensions) $(call import-module,external/Box2D)
