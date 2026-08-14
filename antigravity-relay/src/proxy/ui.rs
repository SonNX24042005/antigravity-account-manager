pub fn get_admin_ui_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="vi" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Quản lý tài khoản - Antigravity</title>
  <style nonce="{{CSP_NONCE}}">
    {{ADMIN_CSS}}
    body { font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }
    .font-mono { font-family: 'JetBrains Mono', monospace; }
    [data-lucide] { display: inline-flex; align-items: center; justify-content: center; font-style: normal; }
    [data-lucide="refresh-cw"]::before { content: '↻'; }
    [data-lucide="plus"]::before { content: '+'; }
    [data-lucide="chevron-down"]::before { content: '⌄'; }
    [data-lucide="globe"]::before { content: '◎'; }
    [data-lucide="key"]::before { content: '⚿'; }
    [data-lucide="x"]::before { content: '×'; }
    [data-lucide="trash-2"]::before { content: '⌫'; }
    [data-lucide="arrow-right-left"]::before { content: '⇄'; }
    [data-lucide="check"]::before { content: '✓'; }
    [data-lucide="user-x"]::before { content: '∅'; }
    .quota-progress { appearance: none; display: block; width: 100%; height: 0.25rem; overflow: hidden; border: 1px solid rgb(39 39 42); border-radius: 9999px; background: rgb(24 24 27); }
    .quota-progress::-webkit-progress-bar { background: rgb(24 24 27); }
    .quota-progress::-webkit-progress-value { background: linear-gradient(to right, #2563eb, #38bdf8); }
    .quota-progress.low::-webkit-progress-value { background: #fbbf24; }
    .quota-progress::-moz-progress-bar { background: linear-gradient(to right, #2563eb, #38bdf8); }
    .quota-progress.low::-moz-progress-bar { background: #fbbf24; }
  </style>
</head>
<body class="bg-[#0b0c10] text-zinc-200 min-h-screen antialiased selection:bg-blue-600/30">
  <!-- Header -->
  <header class="border-b border-zinc-800/80 bg-[#10121a]/80 backdrop-blur sticky top-0 z-40">
    <div class="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-blue-600/10 border border-blue-500/30 flex items-center justify-center text-sm font-semibold text-blue-400">
          A
        </div>
        <div>
          <h1 class="text-sm font-semibold text-zinc-100">Quản lý tài khoản</h1>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2 px-2.5 py-1 rounded-md text-xs font-mono bg-zinc-900/90 border border-zinc-800 text-zinc-300">
          <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
          <span id="relay-address">Đang kết nối...</span>
        </div>
        <button id="refresh-btn" title="Làm mới" class="p-1.5 hover:bg-zinc-800 rounded-md text-zinc-400 hover:text-zinc-200 transition">
          <i data-lucide="refresh-cw" class="w-4 h-4 text-zinc-400"></i>
        </button>
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="max-w-6xl mx-auto px-6 py-8">
    <!-- Action & Summary Bar -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
      <div>
        <h2 class="text-base font-semibold text-zinc-100">Danh sách tài khoản</h2>
        <p class="text-xs text-zinc-400 mt-0.5">Tự động chọn tài khoản có nhiều hạn ngạch nhất theo đúng mô hình đang dùng</p>
      </div>

      <!-- Add Account Dropdown Menu -->
      <div class="relative">
        <button id="add-account-btn" class="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition flex items-center gap-1.5 cursor-pointer">
          <i data-lucide="plus" class="w-3.5 h-3.5"></i>
          Thêm tài khoản
          <i data-lucide="chevron-down" class="w-3.5 h-3.5 ml-0.5 text-blue-200"></i>
        </button>

        <!-- Dropdown Menu -->
        <div id="add-menu" class="absolute right-0 mt-2 w-72 bg-zinc-900 border border-zinc-800 rounded-xl shadow-2xl p-1.5 z-50 hidden">
          <div class="px-2.5 py-1.5 text-[10px] font-medium text-zinc-500 uppercase tracking-wider">
            Chọn phương thức thêm
          </div>
          
          <!-- Option 1: Google OAuth -->
          <button id="oauth-option-btn" class="w-full text-left p-2.5 rounded-lg hover:bg-zinc-800/80 transition flex items-start gap-3 group cursor-pointer">
            <div class="w-7 h-7 rounded-md bg-blue-500/10 border border-blue-500/20 text-blue-400 flex items-center justify-center flex-shrink-0 mt-0.5 group-hover:bg-blue-500 group-hover:text-white transition">
              <i data-lucide="globe" class="w-4 h-4"></i>
            </div>
            <div>
              <div class="text-xs font-semibold text-zinc-200 group-hover:text-white flex items-center gap-1.5">
                Đăng nhập Google
                <span class="text-[9px] px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 font-normal">Khuyên dùng</span>
              </div>
              <div class="text-[10px] text-zinc-400 mt-0.5 leading-snug">
                Tự động xác thực qua trình duyệt và tự động làm mới token
              </div>
            </div>
          </button>

          <!-- Option 2: Direct Token -->
          <button id="direct-option-btn" class="w-full text-left p-2.5 rounded-lg hover:bg-zinc-800/80 transition flex items-start gap-3 group cursor-pointer">
            <div class="w-7 h-7 rounded-md bg-zinc-800 border border-zinc-700/60 text-zinc-400 flex items-center justify-center flex-shrink-0 mt-0.5 group-hover:bg-zinc-700 group-hover:text-zinc-200 transition">
              <i data-lucide="key" class="w-4 h-4"></i>
            </div>
            <div>
              <div class="text-xs font-semibold text-zinc-200 group-hover:text-white">
                Nhập token thủ công
              </div>
              <div class="text-[10px] text-zinc-400 mt-0.5 leading-snug">
                Dán access token hoặc refresh token trực tiếp
              </div>
            </div>
          </button>
        </div>
      </div>
    </div>

    <!-- Quick Stats & Intelligent Routing -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-6">
      <div class="p-3.5 rounded-xl bg-zinc-900/50 border border-zinc-800/80 flex flex-col justify-between">
        <span class="text-xs text-zinc-400">Tổng số tài khoản</span>
        <div class="text-xl font-semibold text-zinc-100 mt-1" id="stat-total">0</div>
      </div>

      <div class="p-3.5 rounded-xl bg-zinc-900/50 border border-zinc-800/80 flex flex-col justify-between">
        <span class="text-xs text-zinc-400">Tài khoản đang dùng</span>
        <div class="text-xl font-semibold text-blue-400 mt-1 truncate" id="stat-active">Chưa có</div>
      </div>

      <!-- Smart Model Routing Preference Card -->
      <div class="p-3.5 rounded-xl bg-zinc-900/50 border border-zinc-800/80 flex flex-col justify-between">
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs text-zinc-400">Chế độ tự động chọn</span>
          <select id="pref-select" class="bg-zinc-950 border border-zinc-800 text-[11px] text-zinc-200 rounded-md px-2 py-0.5 focus:outline-none focus:border-blue-500">
            <option value="auto">Tự động (Theo mô hình vừa dùng)</option>
            <option value="gemini">Luôn ưu tiên Gemini</option>
            <option value="claude_gpt">Luôn ưu tiên Claude & GPT</option>
          </select>
        </div>
        <div class="mt-2 flex items-center justify-between">
          <div class="flex items-center gap-1.5 text-xs font-medium text-zinc-200">
            <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
            <span id="detected-model-label">Đang phát hiện...</span>
          </div>
          <span id="detected-source-tag" class="text-[10px] text-zinc-500 truncate max-w-[150px]" title=""></span>
        </div>
      </div>
    </div>

    <!-- Accounts Grid -->
    <div id="accounts-grid" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div class="col-span-full py-12 text-center text-zinc-500 text-sm">
        Đang tải dữ liệu...
      </div>
    </div>
  </main>

  <!-- Modal: Add Direct Token -->
  <div id="add-modal" class="fixed inset-0 bg-zinc-950/80 backdrop-blur-sm z-50 flex items-center justify-center hidden p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl p-5 w-full max-w-md shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-sm font-semibold text-zinc-100">Nhập token thủ công</h3>
          <p class="text-[11px] text-zinc-400 mt-0.5">Dán thông tin token của tài khoản Google</p>
        </div>
        <button id="close-add-modal-btn" class="text-zinc-400 hover:text-zinc-200">
          <i data-lucide="x" class="w-4 h-4"></i>
        </button>
      </div>
      <div class="space-y-3.5">
        <div>
          <label class="block text-xs font-medium text-zinc-400 mb-1">Email Google</label>
          <input id="input-email" type="email" placeholder="user@gmail.com" class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs focus:outline-none focus:border-blue-500 text-zinc-200">
        </div>
        <div>
          <label class="block text-xs font-medium text-zinc-400 mb-1">Access token</label>
          <textarea id="input-access-token" rows="3" placeholder="ya29.a0..." class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs focus:outline-none focus:border-blue-500 font-mono text-zinc-200"></textarea>
        </div>
        <div>
          <label class="block text-xs font-medium text-zinc-400 mb-1">Refresh token (tùy chọn)</label>
          <input id="input-refresh-token" type="text" placeholder="1//04..." class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs focus:outline-none focus:border-blue-500 font-mono text-zinc-200">
        </div>
      </div>
      <div class="flex items-center justify-between mt-5 pt-3 border-t border-zinc-800">
        <button id="modal-oauth-btn" class="text-xs text-blue-400 hover:text-blue-300 transition cursor-pointer">
          Hoặc đăng nhập Google
        </button>
        <div class="flex items-center gap-2">
          <button id="cancel-add-btn" class="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-300 cursor-pointer">Đóng</button>
          <button id="save-add-btn" class="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition cursor-pointer">Lưu tài khoản</button>
        </div>
      </div>
    </div>
  </div>

  <script nonce="{{CSP_NONCE}}">
    let SESSION_READY = false;

    async function apiFetch(url, options) {
      const request = options ? { ...options } : {};
      request.credentials = 'same-origin';
      const response = await window.fetch(url, request);
      if (response.status === 401) {
        SESSION_READY = false;
        showSessionHelp();
        throw new Error('Phiên quản trị đã hết hạn. Hãy chạy lại lệnh agyr.');
      }
      return response;
    }

    async function establishBrowserSession() {
      const fragment = new URLSearchParams(window.location.hash.slice(1));
      const bootstrapToken = fragment.get('bootstrap');
      if (bootstrapToken) {
        window.history.replaceState(null, '', window.location.pathname + window.location.search);
        const exchange = await window.fetch('/api/session/exchange', {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ bootstrap_token: bootstrapToken })
        });
        if (!exchange.ok) {
          return false;
        }
      }

      const health = await window.fetch('/api/health', { credentials: 'same-origin' });
      return health.ok;
    }

    function showSessionHelp() {
      document.getElementById('accounts-grid').innerHTML = `
        <div class="col-span-full py-16 text-center rounded-xl border border-dashed border-zinc-800 bg-zinc-900/20">
          <p class="text-zinc-300 text-sm font-medium">Cần mở phiên quản trị an toàn</p>
          <p class="text-xs text-zinc-500 mt-1">Chạy <code class="font-mono text-blue-400">agyr</code> trong terminal để mở lại tự động.</p>
        </div>
      `;
    }

    function escapeHtml(value) {
      return String(value ?? '').replace(/[&<>"']/g, (character) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
      })[character]);
    }

    function toggleAddMenu() {
      const menu = document.getElementById('add-menu');
      menu.classList.toggle('hidden');
    }

    function handleOptionOAuth() {
      document.getElementById('add-menu').classList.add('hidden');
      startOAuthLogin();
    }

    function handleOptionDirectToken() {
      document.getElementById('add-menu').classList.add('hidden');
      openDirectAddModal();
    }

    // Close dropdown on outside click
    document.addEventListener('click', (e) => {
      const btn = document.getElementById('add-account-btn');
      const menu = document.getElementById('add-menu');
      if (btn && menu && !btn.contains(e.target) && !menu.contains(e.target)) {
        menu.classList.add('hidden');
      }
    });

    async function fetchPreference() {
      try {
        const res = await apiFetch('/api/preference');
        const data = await res.json();
        const select = document.getElementById('pref-select');
        if (select) {
          select.value = data.preference;
        }

        const label = document.getElementById('detected-model-label');
        if (label) {
          if (data.preference === 'gemini') {
            label.innerText = 'Ưu tiên Gemini';
          } else if (data.preference === 'claude_gpt') {
            label.innerText = 'Ưu tiên Claude & GPT';
          } else {
            label.innerText = data.detected_category === 'claude_gpt' ? 'Đang dùng: Claude & GPT' : 'Đang dùng: Gemini Models';
          }
        }

        const tag = document.getElementById('detected-source-tag');
        if (tag && data.last_detected_source) {
          tag.innerText = data.last_detected_source;
          tag.title = data.last_detected_source;
        }
      } catch (e) {
        console.error('Failed to fetch preference:', e);
      }
    }

    async function changePreference(prefVal) {
      try {
        const res = await apiFetch('/api/preference', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ preference: prefVal })
        });
        if (res.ok) {
          fetchPreference();
          fetchAccounts();
        }
      } catch (e) {
        alert('Lỗi cập nhật cấu hình: ' + e);
      }
    }

    async function fetchAccounts() {
      try {
        const res = await apiFetch('/api/accounts');
        const accounts = await res.json();
        renderAccounts(accounts);
      } catch (err) {
        console.error('Failed to fetch accounts:', err);
      }
    }

    function renderAccounts(accounts) {
      document.getElementById('stat-total').innerText = accounts.length;
      const activeAcc = accounts.find(a => a.is_active);
      document.getElementById('stat-active').innerText = activeAcc ? activeAcc.email : 'Chưa chọn';

      const container = document.getElementById('accounts-grid');
      if (accounts.length === 0) {
        container.innerHTML = `
          <div class="col-span-full py-16 text-center rounded-xl border border-dashed border-zinc-800 bg-zinc-900/20">
            <i data-lucide="user-x" class="w-8 h-8 text-zinc-600 mx-auto mb-2"></i>
            <p class="text-zinc-400 text-sm font-medium">Chưa có tài khoản nào</p>
            <p class="text-xs text-zinc-500 mt-0.5">Bấm nút "Thêm tài khoản" ở trên để bắt đầu thêm tài khoản.</p>
          </div>
        `;
        return;
      }

      container.innerHTML = accounts.map(acc => {
        const isActive = acc.is_active;
        const safeEmail = escapeHtml(acc.email);
        const safeId = escapeHtml(acc.id);
        const safeInitials = escapeHtml(acc.email.substring(0, 2).toUpperCase());

        return `
          <div class="p-4 rounded-xl transition flex flex-col justify-between ${
            isActive
              ? 'bg-[#121626] border border-blue-500/50 shadow-[0_0_24px_rgba(59,130,246,0.06)]'
              : 'bg-zinc-900/60 border border-zinc-800/80 hover:border-zinc-700'
          }">
            <div>
              <div class="flex items-start justify-between gap-2 mb-3.5">
                <div class="flex items-center gap-2.5 min-w-0">
                  <div class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold flex-shrink-0 ${
                    isActive
                      ? 'bg-blue-950/80 text-blue-300 border border-blue-500/40'
                      : 'bg-zinc-800 text-zinc-300 border border-zinc-700/60'
                  }">
                    ${safeInitials}
                  </div>
                  <div class="min-w-0">
                    <h3 class="font-medium text-xs text-zinc-100 truncate" title="${safeEmail}">${safeEmail}</h3>
                    <p class="text-[10px] text-zinc-500 font-mono mt-0.5 truncate">id: ${escapeHtml(acc.id.substring(0, 8))}</p>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  ${
                    isActive
                      ? `<span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-medium bg-blue-500/10 text-blue-400 border border-blue-500/20">
                          <span class="w-1.5 h-1.5 rounded-full bg-blue-400"></span>
                          Đang dùng
                        </span>`
                      : `<span class="inline-flex items-center gap-1.5 text-[10px] font-medium text-zinc-400 px-1 py-0.5">
                          <span class="w-1.5 h-1.5 rounded-full bg-zinc-600"></span>
                          Sẵn sàng
                        </span>`
                  }
                  <button data-account-id="${safeId}" data-account-email="${safeEmail}" title="Xóa tài khoản" class="delete-account-btn p-1 hover:bg-zinc-800 text-zinc-500 hover:text-red-400 rounded-md transition flex items-center justify-center cursor-pointer">
                    <i data-lucide="trash-2" class="w-3.5 h-3.5"></i>
                  </button>
                </div>
              </div>

              <!-- Quota Breakdown -->
              <div class="space-y-2.5 mt-3 pt-3 border-t border-zinc-800/60">
                ${(acc.quota_groups && acc.quota_groups.length > 0) ? acc.quota_groups.map(g => `
                  <div class="space-y-1.5">
                    <div class="text-[11px] font-medium text-zinc-300 flex items-center gap-1">
                      <span>${escapeHtml(g.name)}</span>
                    </div>
                    <div class="space-y-1.5 bg-zinc-950/60 p-2 rounded-lg border border-zinc-800/60">
                      ${g.buckets.map(b => {
                        let pct = Number.isFinite(b.remaining_percentage)
                          ? Math.max(0, Math.min(100, Math.round(b.remaining_percentage)))
                          : 0;
                        const resetInfo = getResetDisplay(b.reset_time, b.window);
                        if (resetInfo.isExpired) {
                          pct = 100;
                        }
                        const isLow = pct < 20;
                        return `
                        <div>
                          <div class="flex justify-between text-[10px] text-zinc-400 mb-0.5">
                            <span>${b.window === 'FIVE_HOUR' ? 'Hạn ngạch 5 giờ' : (b.window === 'WEEKLY' ? 'Hạn ngạch tuần' : escapeHtml(b.window))}</span>
                            <span class="font-mono ${isLow ? 'text-amber-400' : 'text-blue-400'} font-medium">${pct}%</span>
                          </div>
                          <progress class="quota-progress ${isLow ? 'low' : ''}" max="100" value="${pct}">${pct}%</progress>
                          ${resetInfo.text ? `
                            <div class="text-[9px] text-zinc-500 mt-0.5 font-mono">
                              ${escapeHtml(resetInfo.text)}
                            </div>
                          ` : ''}
                        </div>
                      `}).join('')}
                    </div>
                  </div>
                `).join('') : `
                  <div class="bg-zinc-950/60 p-2 rounded-lg border border-zinc-800/60">
                    <div class="flex justify-between text-[10px] text-zinc-400 mb-0.5">
                      <span>Hạn ngạch khả dụng</span>
                      <span class="font-mono font-medium text-blue-400">${Number.isFinite(acc.quota_percentage) ? Math.max(0, Math.min(100, Math.round(acc.quota_percentage))) : 0}%</span>
                    </div>
                    <progress class="quota-progress" max="100" value="${Number.isFinite(acc.quota_percentage) ? Math.max(0, Math.min(100, Math.round(acc.quota_percentage))) : 0}"></progress>
                  </div>
                `}
              </div>
            </div>

            <!-- Action Button -->
            <div class="mt-4 pt-3 border-t border-zinc-800/60">
              ${isActive ? `
                <div class="w-full py-1.5 px-2 text-center text-xs font-medium rounded-lg bg-blue-500/10 text-blue-400 border border-blue-500/20 flex items-center justify-center gap-1.5">
                  <i data-lucide="check" class="w-3.5 h-3.5"></i> Đang hoạt động
                </div>
              ` : `
                <button data-account-id="${safeId}" class="switch-account-btn w-full py-1.5 px-2 text-center text-xs font-medium rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition duration-150 flex items-center justify-center gap-1.5 cursor-pointer">
                  <i data-lucide="arrow-right-left" class="w-3.5 h-3.5 text-zinc-400"></i> Chuyển sang tài khoản này
                </button>
              `}
            </div>
          </div>
        `;
      }).join('');
    }

    async function switchAccount(accountId) {
      try {
        const res = await apiFetch('/api/accounts/switch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ account_id: accountId })
        });
        const data = await res.json();
        if (res.ok) {
          fetchAccounts();
        } else {
          alert('Lỗi: ' + data.error);
        }
      } catch (e) {
        alert('Lỗi kết nối: ' + e.message);
      }
    }

    async function deleteAccount(accountId, email) {
      if (!confirm(`Bạn có chắc chắn muốn xóa tài khoản "${email}" không?`)) {
        return;
      }
      try {
        const res = await apiFetch('/api/accounts/delete', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ account_id: accountId })
        });
        const data = await res.json();
        if (res.ok) {
          fetchAccounts();
        } else {
          alert('Lỗi: ' + (data.error || 'Không thể xóa tài khoản'));
        }
      } catch (e) {
        alert('Lỗi kết nối: ' + e.message);
      }
    }

    function getResetDisplay(resetTimeStr, window) {
      if (!resetTimeStr) {
        return { text: window === 'FIVE_HOUR' ? 'Chu kỳ: 5 giờ (đầy đủ)' : '', isExpired: false };
      }
      try {
        const d = new Date(resetTimeStr);
        const now = new Date();
        const diffMs = d - now;
        
        const hours = d.getHours().toString().padStart(2, '0');
        const mins = d.getMinutes().toString().padStart(2, '0');
        const day = d.getDate().toString().padStart(2, '0');
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        if (diffMs <= 0) {
          return {
            text: window === 'FIVE_HOUR' ? 'Đã hồi phục (100% - Chu kỳ 5h)' : 'Đã hồi phục (100%)',
            isExpired: true
          };
        }

        const diffMinutesTotal = Math.floor(diffMs / (1000 * 60));
        const diffHours = Math.floor(diffMinutesTotal / 60);
        const diffMins = diffMinutesTotal % 60;
        const diffDays = Math.floor(diffHours / 24);

        let timeStr = '';
        if (diffDays > 0) {
          timeStr = `Reset: ${hours}:${mins} (${day}/${month}, còn ${diffDays}d ${diffHours % 24}h)`;
        } else if (diffHours > 0) {
          timeStr = `Reset: ${hours}:${mins} (còn ${diffHours}h ${diffMins}m)`;
        } else {
          timeStr = `Reset: ${hours}:${mins} (còn ${diffMins}m)`;
        }

        return { text: timeStr, isExpired: false };
      } catch (e) {
        return { text: '', isExpired: false };
      }
    }

    async function startOAuthLogin() {
      try {
        const res = await apiFetch('/api/accounts/oauth/start');
        const data = await res.json();
        if (data.auth_url) {
          window.open(data.auth_url, '_blank');
        }
      } catch (err) {
        alert('Lỗi khi lấy URL OAuth: ' + err);
      }
    }

    function openDirectAddModal() {
      document.getElementById('add-modal').classList.remove('hidden');
    }

    function closeDirectAddModal() {
      document.getElementById('add-modal').classList.add('hidden');
    }

    async function submitDirectAdd() {
      const email = document.getElementById('input-email').value;
      const access_token = document.getElementById('input-access-token').value;
      const refresh_token = document.getElementById('input-refresh-token').value;

      if (!email || !access_token) {
        alert('Vui lòng nhập Email và Access token');
        return;
      }

      try {
        const res = await apiFetch('/api/accounts/add', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, access_token, refresh_token })
        });
        if (res.ok) {
          closeDirectAddModal();
          fetchAccounts();
        } else {
          alert('Lỗi khi thêm tài khoản');
        }
      } catch (err) {
        alert('Lỗi: ' + err);
      }
    }

    document.getElementById('refresh-btn').addEventListener('click', () => {
      fetchAccounts();
      fetchPreference();
    });
    document.getElementById('add-account-btn').addEventListener('click', toggleAddMenu);
    document.getElementById('oauth-option-btn').addEventListener('click', handleOptionOAuth);
    document.getElementById('direct-option-btn').addEventListener('click', handleOptionDirectToken);
    document.getElementById('pref-select').addEventListener('change', (event) => changePreference(event.target.value));
    document.getElementById('close-add-modal-btn').addEventListener('click', closeDirectAddModal);
    document.getElementById('cancel-add-btn').addEventListener('click', closeDirectAddModal);
    document.getElementById('save-add-btn').addEventListener('click', submitDirectAdd);
    document.getElementById('modal-oauth-btn').addEventListener('click', () => {
      closeDirectAddModal();
      startOAuthLogin();
    });
    document.getElementById('accounts-grid').addEventListener('click', (event) => {
      const deleteButton = event.target.closest('.delete-account-btn');
      if (deleteButton) {
        deleteAccount(deleteButton.dataset.accountId, deleteButton.dataset.accountEmail);
        return;
      }
      const switchButton = event.target.closest('.switch-account-btn');
      if (switchButton) {
        switchAccount(switchButton.dataset.accountId);
      }
    });

    document.getElementById('relay-address').innerText = window.location.host;

    async function initializeDashboard() {
      try {
        SESSION_READY = await establishBrowserSession();
      } catch (error) {
        console.error('Failed to establish browser session:', error);
        SESSION_READY = false;
      }
      if (!SESSION_READY) {
        showSessionHelp();
        return;
      }
      fetchAccounts();
      fetchPreference();
    }

    initializeDashboard();
    setInterval(() => {
      if (SESSION_READY) {
        fetchAccounts();
        fetchPreference();
      }
    }, 5000);
  </script>
</body>
</html>
"#
}

#[cfg(test)]
mod tests {
    use super::get_admin_ui_html;

    #[test]
    fn admin_ui_contains_no_embedded_secret_or_remote_script() {
        let html = get_admin_ui_html();
        assert!(!html.contains("MASTER_KEY"));
        assert!(!html.contains("<script src="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onchange="));
        assert!(!html.contains("sessionStorage"));
        assert!(!html.contains("window.prompt"));
        assert!(html.contains("/api/session/exchange"));
    }
}
