package com.rucraft;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class RucraftMod implements ModInitializer {
    public static final String MOD_ID = "rucraft";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        String message = RustBridge.hello();
        int result = RustBridge.add(20, 22);

        LOGGER.info("{}; native says 20 + 22 = {}", message, result);
    }
}
