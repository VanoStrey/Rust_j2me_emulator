package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.game.GameCanvas;

public class Fixture8PaintAndActiveRender extends MIDlet {
    public static class HybridCanvas extends GameCanvas {
        public HybridCanvas() { super(false); }
        public void paint(Graphics g) {
            // Passive paint draw
            g.setColor(0x00FF00FF); // Magenta
            g.fillRect(0, 0, 50, 50);
        }
        public void runActive() {
            Graphics g = getGraphics();
            g.setColor(0x0000FFFF); // Cyan
            g.fillRect(100, 100, 50, 50);
            flushGraphics();
        }
    }

    protected void startApp() {
        HybridCanvas hc = new HybridCanvas();
        Display.getDisplay(this).setCurrent(hc);
        hc.runActive();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
