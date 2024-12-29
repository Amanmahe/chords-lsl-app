"use client";

import React, { useRef, useState } from 'react';
import dynamic from 'next/dynamic';
const ClientOnlyPage = dynamic(() => import('./main'), { ssr: false });

const Page = () => {
  return (
    <div>
      <ClientOnlyPage />
    </div>
  );
};

export default Page;