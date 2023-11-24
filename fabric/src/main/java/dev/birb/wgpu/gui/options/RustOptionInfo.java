package dev.birb.wgpu.gui.options;

import lombok.Getter;
import lombok.Setter;

@Getter
@Setter
public class RustOptionInfo {
    private String text;
    private String desc;
    private boolean needsRestart;
    private String[] variants = new String[]{};
}
