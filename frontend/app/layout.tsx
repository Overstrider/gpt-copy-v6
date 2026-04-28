import type { Metadata } from "next";

import "./globals.css";

export const metadata: Metadata = {
  title: "gpt-copy-v6",
  description: "ChatGPT-style application backed by the gpt-copy-v6 API"
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
