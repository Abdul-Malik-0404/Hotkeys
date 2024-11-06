#include <windows.h>
#include <iostream>

void simulateKeyPress(WORD vk) {
    INPUT input[2] = {};
    input[0].type = INPUT_KEYBOARD;
    input[0].ki.wVk = vk;
    input[1].type = INPUT_KEYBOARD;
    input[1].ki.wVk = vk;
    input[1].ki.dwFlags = KEYEVENTF_KEYUP;
    SendInput(2, input, sizeof(INPUT));
}

void playPause() {
    simulateKeyPress(VK_MEDIA_PLAY_PAUSE);
}

void nextTrack() {
    simulateKeyPress(VK_MEDIA_NEXT_TRACK);
}

void prevTrack() {
    simulateKeyPress(VK_MEDIA_PREV_TRACK);
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    // Register global hotkeys
    RegisterHotKey(NULL, 1, MOD_NOREPEAT, VK_F1);  // Play/Pause
    RegisterHotKey(NULL, 2, MOD_NOREPEAT, VK_F2);  // Next Track
    RegisterHotKey(NULL, 3, MOD_NOREPEAT, VK_F3);  // Previous Track

    MSG msg = { 0 };
    while (GetMessage(&msg, NULL, 0, 0)) {
        if (msg.message == WM_HOTKEY) {
            switch (msg.wParam) {
            case 1:
                playPause();
                break;
            case 2:
                prevTrack();
                break;
            case 3:
                nextTrack();
                break;
            }
        }
    }

    return 0;
}
