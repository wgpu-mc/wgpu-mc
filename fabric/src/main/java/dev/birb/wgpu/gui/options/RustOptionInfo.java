package dev.birb.wgpu.gui.options;

public class RustOptionInfo {
    private String text;
    private String desc;
    private boolean needsRestart;
    private String[] variants = new String[]{};

    public String[] getVariants() {
        return variants;
    }

    public void setVariants(String[] variants) {
        this.variants = variants;
    }


    public String getText() {
        return text;
    }

    public void setText(String text) {
        this.text = text;
    }

    public String getDesc() {
        return desc;
    }

    public void setDesc(String desc) {
        this.desc = desc;
    }

    public boolean needsRestart() {
        return needsRestart;
    }

    public void setNeedsRestart(boolean needsRestart) {
        this.needsRestart = needsRestart;
    }
}
