'use client';

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useMessage(num: number) {
    const [message, setMessage] = useState<string>('');

    async function fetchMessage() {
      try {
        const response = await invoke('my_custom_command', { num });
        setMessage(response as string);
      } catch (error) {
        console.error('Error invoking Tauri command:', error);
      }
    }
    useEffect(() => {
      fetchMessage();
    }, [num]);

    return { message, fetchMessage };
}
