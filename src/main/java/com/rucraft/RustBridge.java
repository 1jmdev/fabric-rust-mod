package com.rucraft;

public final class RustBridge {
    static {
        NativeLoader.load();
    }

    private RustBridge() {
    }

    public static native String hello();

    public static native int add(int left, int right);
}
