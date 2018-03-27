#include <string>
#include <map>
#include <nan.h>
#include "indy_core.h"

char* copyCStr(const char* original){
    size_t len = strlen(original);
    char* dest = new char[len + 1];
    strncpy(dest, original, len);
    dest[len] = '\0';
    return dest;
}

char* copyBuffer(const indy_u8_t* data, indy_u32_t len){
    char* dest = (char*)malloc(len * sizeof(char));
    memcpy(dest, data, len);
    return dest;
}

enum IndyCallbackType {
    CB_NONE,
    CB_STRING,
    CB_BOOLEAN,
    CB_HANDLE,
    CB_BUFFER,
    CB_STRING_BUFFER,
    CB_STRING_STRING
};

class IndyCallback : public Nan::AsyncResource {
  public:
    IndyCallback(v8::Local<v8::Function> callback_) : Nan::AsyncResource("IndyCallback") {
        callback.Reset(callback_);
        uvHandle.data = this;
        type = CB_NONE;
        next_handle++;
        handle = next_handle;
        icbmap[handle] = this;
        uv_async_init(uv_default_loop(), &uvHandle, onMainLoopReentry);
        str0 = nullptr;
        str1 = nullptr;
        buffer0data = nullptr;
    }

    ~IndyCallback() {
        callback.Reset();
        delete str0;
        delete str1;
        // NOTE: do not `free(buffer0data)` b/c Nan::NewBuffer assumes ownership and node's garbage collector will free it.
    }

    void cbNone(indy_error_t xerr){
        send(xerr);
    }

    void cbString(indy_error_t xerr, const char* str){
        if(xerr == 0){
          type = CB_STRING;
          str0 = copyCStr(str);
        }
        send(xerr);
    }

    void cbStringString(indy_error_t xerr, const char* strA, const char* strB){
        if(xerr == 0){
          type = CB_STRING_STRING;
          str0 = copyCStr(strA);
          str1 = copyCStr(strB);
        }
        send(xerr);
    }

    void cbBoolean(indy_error_t xerr, bool b){
        if(xerr == 0){
          type = CB_BOOLEAN;
          bool0 = b;
        }
        send(xerr);
    }

    void cbHandle(indy_error_t xerr, indy_handle_t h){
        if(xerr == 0){
          type = CB_HANDLE;
          handle0 = h;
        }
        send(xerr);
    }

    void cbBuffer(indy_error_t xerr, const indy_u8_t* data, indy_u32_t len){
        if(xerr == 0){
            type = CB_BUFFER;
            buffer0data = copyBuffer(data, len);
            buffer0len = len;
        }
        send(xerr);
    }

    void cbStringBuffer(indy_error_t xerr, const char* str, const indy_u8_t* data, indy_u32_t len){
        if(xerr == 0){
            type = CB_STRING_BUFFER;
            str0 = copyCStr(str);
            buffer0data = copyBuffer(data, len);
            buffer0len = len;
        }
        send(xerr);
    }


    indy_handle_t handle;

    static IndyCallback* getCallback(indy_handle_t handle){
        if(icbmap.count(handle) == 0){
            return nullptr;
        }
        return icbmap[handle];
    }

  private:

    static indy_handle_t next_handle;
    static std::map<indy_handle_t, IndyCallback*> icbmap;

    Nan::Persistent<v8::Function> callback;
    uv_async_t uvHandle;

    IndyCallbackType type;
    indy_error_t err;
    const char* str0;
    const char* str1;
    bool bool0;
    indy_handle_t handle0;
    char*    buffer0data;
    uint32_t buffer0len;

    void send(indy_error_t xerr){
        err = xerr;
        uv_async_send(&uvHandle);
    }

    inline static NAUV_WORK_CB(onMainLoopReentry) {
        Nan::HandleScope scope;

        IndyCallback* icb = static_cast<IndyCallback*>(async->data);
        icbmap.erase(icb->handle);

        int argc = icb->type == CB_NONE ? 1 : 2;

        v8::Local<v8::Array> tuple;
        v8::Local<v8::Value> argv[argc];
        argv[0] = Nan::New<v8::Number>(icb->err);
        switch(icb->type){
            case CB_NONE:
                // nothing
                break;
            case CB_STRING:
                argv[1] = Nan::New<v8::String>(icb->str0).ToLocalChecked();
                break;
            case CB_BOOLEAN:
                argv[1] = Nan::New<v8::Boolean>(icb->bool0);
                break;
            case CB_HANDLE:
                argv[1] = Nan::New<v8::Number>(icb->handle0);
                break;
            case CB_BUFFER:
                argv[1] = Nan::NewBuffer(icb->buffer0data, icb->buffer0len).ToLocalChecked();
                break;
            case CB_STRING_BUFFER:
                tuple = Nan::New<v8::Array>();
                tuple->Set(0, Nan::New<v8::String>(icb->str0).ToLocalChecked());
                tuple->Set(0, Nan::NewBuffer(icb->buffer0data, icb->buffer0len).ToLocalChecked());
                argv[1] = tuple;
                break;
            case CB_STRING_STRING:
                tuple = Nan::New<v8::Array>();
                tuple->Set(0, Nan::New<v8::String>(icb->str0).ToLocalChecked());
                tuple->Set(1, Nan::New<v8::String>(icb->str1).ToLocalChecked());
                argv[1] = tuple;
                break;
        }

        v8::Local<v8::Object> target = Nan::New<v8::Object>();
        v8::Local<v8::Function> callback = Nan::New(icb->callback);
        icb->runInAsyncScope(target, callback, argc, argv);

        uv_close(reinterpret_cast<uv_handle_t*>(&icb->uvHandle), onUvHandleClose);
    }

    inline static void onUvHandleClose(uv_handle_t* async) {
        Nan::HandleScope scope;
        IndyCallback* icb = static_cast<IndyCallback*>(async->data);
        delete icb;
    }
};

std::map<indy_handle_t, IndyCallback*> IndyCallback::icbmap;
indy_handle_t IndyCallback::next_handle = 0;

void indyCalled(IndyCallback* icb, indy_error_t res) {
    if(res == 0) {
        return;
    }
    icb->cbNone(res);
}

// Now inject the generated C++ code (see /codegen/cpp.js)
#include "indy_codegen.h"
