"use client";

import Link from "next/link";
import { Shield, ArrowLeft } from "lucide-react";

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-slate-900 text-slate-200">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-900">
        <div className="max-w-4xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <Link href="/" className="flex items-center gap-3 hover:opacity-80 transition-opacity">
              <Shield className="text-emerald-400" size={32} />
              <span className="text-2xl font-bold text-white">Veridion Nexus</span>
            </Link>
            <Link
              href="/"
              className="flex items-center gap-2 text-slate-400 hover:text-white transition-colors"
            >
              <ArrowLeft size={16} />
              <span>Back</span>
            </Link>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto px-6 py-10">
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-8 space-y-8">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">Terms of Service</h1>
            <p className="text-slate-400">Last updated: {new Date().toLocaleDateString()}</p>
          </div>

          <div className="space-y-6 text-slate-300">
            <section>
              <h2 className="text-xl font-semibold text-white mb-3">Limitation of Liability & No Legal Advice</h2>
              
              <div className="space-y-4">
                <div>
                  <h3 className="text-lg font-semibold text-slate-100 mb-2">1. Nature of Service</h3>
                  <p className="text-sm leading-relaxed">
                    The Veridion Nexus platform ("Service") provides technical controls and automation features designed to assist organizations in managing their compliance obligations under regulations such as GDPR, DORA, and the EU AI Act. <strong className="text-slate-100">The Service does not constitute, and is not a substitute for, professional legal advice.</strong> Veridion does not guarantee that the use of the Service will ensure your compliance with any specific law or regulation.
                  </p>
                </div>

                <div>
                  <h3 className="text-lg font-semibold text-slate-100 mb-2">2. No Guarantee of Results</h3>
                  <p className="text-sm leading-relaxed">
                    While Veridion Nexus aims to mitigate risks through features like "Sovereign Lock" and "Shadow Mode", we do not guarantee that these technical measures will be 100% effective against all forms of data exfiltration, cyber threats, or regulatory scrutiny. You acknowledge that regulatory compliance is a complex legal matter that depends on your specific internal processes, not just software.
                  </p>
                </div>

                <div>
                  <h3 className="text-lg font-semibold text-slate-100 mb-2">3. Limitation of Liability</h3>
                  <p className="text-sm leading-relaxed">
                    To the maximum extent permitted by law, Veridion shall not be liable for any indirect, incidental, special, consequential, or punitive damages, including but not limited to <strong className="text-slate-100">regulatory fines, penalties, loss of profits, or data loss</strong>, arising out of or in connection with your use of the Service.
                  </p>
                </div>

                <div>
                  <h3 className="text-lg font-semibold text-slate-100 mb-2">4. User Responsibility</h3>
                  <p className="text-sm leading-relaxed">
                    You are solely responsible for configuring the Service (including "Shadow Mode" and blocking rules) to meet your specific legal requirements. Veridion is not responsible for data transfers or policy violations that occur due to misconfiguration or manual overrides authorized by your administrators.
                  </p>
                </div>
              </div>
            </section>

            <div className="border-t border-slate-700 pt-6">
              <p className="text-xs text-slate-500">
                By using Veridion Nexus, you acknowledge that you have read, understood, and agree to be bound by these Terms of Service.
              </p>
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800 py-6 mt-10">
        <div className="max-w-4xl mx-auto px-6 text-center text-xs text-slate-500">
          <p>
            Veridion Nexus provides technical governance tools to assist with regulatory compliance. It does not constitute legal advice. You remain solely responsible for your compliance with GDPR, DORA, and the EU AI Act.
          </p>
          <p className="mt-2">© 2024 Veridion Nexus. All rights reserved.</p>
        </div>
      </footer>
    </div>
  );
}

