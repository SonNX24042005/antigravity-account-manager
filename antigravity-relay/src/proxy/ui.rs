pub fn get_admin_ui_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="vi" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Quản lý tài khoản - Antigravity</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/lucide@latest"></script>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');
    body { font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }
    .font-mono { font-family: 'JetBrains Mono', monospace; }
  </style>
</head>
<body class="bg-zinc-950 text-zinc-200 min-h-screen antialiased selection:bg-zinc-800">
  <!-- Header -->
  <header class="border-b border-zinc-800/80 bg-zinc-900/30 backdrop-blur sticky top-0 z-40">
    <div class="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center text-sm font-semibold text-zinc-100">
          A
        </div>
        <div>
          <h1 class="text-sm font-semibold text-zinc-100">Quản lý tài khoản</h1>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2 px-2.5 py-1 rounded-md text-xs font-mono bg-zinc-900 border border-zinc-800 text-zinc-400">
          <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
          127.0.0.1:8045
        </div>
        <button onclick="fetchAccounts()" title="Làm mới" class="p-1.5 hover:bg-zinc-800 rounded-md text-zinc-400 hover:text-zinc-200 transition">
          <i data-lucide="refresh-cw" class="w-4 h-4"></i>
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
        <p class="text-xs text-zinc-400 mt-0.5">Tự động chọn tài khoản có nhiều hạn ngạch nhất khi chạy agy</p>
      </div>
      <div class="flex items-center gap-2.5">
        <button onclick="openDirectAddModal()" class="px-3 py-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-xs font-medium text-zinc-300 transition flex items-center gap-1.5">
          <i data-lucide="key" class="w-3.5 h-3.5 text-zinc-400"></i>
          Thêm token
        </button>
        <button onclick="startOAuthLogin()" class="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition flex items-center gap-1.5">
          <i data-lucide="plus" class="w-3.5 h-3.5"></i>
          Đăng nhập Google
        </button>
      </div>
    </div>

    <!-- Quick Stats -->
    <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-6">
      <div class="p-3.5 rounded-xl bg-zinc-900/40 border border-zinc-800/80">
        <span class="text-xs text-zinc-400">Tổng số tài khoản</span>
        <div class="text-xl font-semibold text-zinc-100 mt-1" id="stat-total">0</div>
      </div>
      <div class="p-3.5 rounded-xl bg-zinc-900/40 border border-zinc-800/80">
        <span class="text-xs text-zinc-400">Tài khoản đang dùng</span>
        <div class="text-xl font-semibold text-zinc-100 mt-1" id="stat-active">0</div>
      </div>
      <div class="p-3.5 rounded-xl bg-zinc-900/40 border border-zinc-800/80 col-span-2 sm:col-span-1">
        <span class="text-xs text-zinc-400">Chế độ chọn tự động</span>
        <div class="text-xs font-medium text-zinc-300 mt-2 flex items-center gap-1.5">
          <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
          Ưu tiên hạn ngạch 5h Gemini
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
        <h3 class="text-sm font-semibold text-zinc-100">Thêm token trực tiếp</h3>
        <button onclick="closeDirectAddModal()" class="text-zinc-400 hover:text-zinc-200">
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
      <div class="flex items-center justify-end gap-2 mt-5">
        <button onclick="closeDirectAddModal()" class="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-300">Đóng</button>
        <button onclick="submitDirectAdd()" class="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition">Lưu tài khoản</button>
      </div>
    </div>
  </div>

  <script>
    lucide.createIcons();

    async function fetchAccounts() {
      try {
        const res = await fetch('/api/accounts');
        const accounts = await res.json();
        renderAccounts(accounts);
      } catch (err) {
        console.error('Failed to fetch accounts:', err);
      }
    }

    function renderAccounts(accounts) {
      document.getElementById('stat-total').innerText = accounts.length;
      const activeCount = accounts.filter(a => a.is_active).length;
      document.getElementById('stat-active').innerText = activeCount;

      const container = document.getElementById('accounts-grid');
      if (accounts.length === 0) {
        container.innerHTML = `
          <div class="col-span-full py-16 text-center rounded-xl border border-dashed border-zinc-800 bg-zinc-900/20">
            <i data-lucide="user-x" class="w-8 h-8 text-zinc-600 mx-auto mb-2"></i>
            <p class="text-zinc-400 text-sm font-medium">Chưa có tài khoản nào</p>
            <p class="text-xs text-zinc-500 mt-0.5">Bấm nút "Đăng nhập Google" ở trên để thêm tài khoản.</p>
          </div>
        `;
        lucide.createIcons();
        return;
      }

      container.innerHTML = accounts.map(acc => {
        const isActive = acc.is_active;

        return `
          <div class="p-4 rounded-xl transition flex flex-col justify-between ${
            isActive
              ? 'bg-zinc-900/70 border border-blue-500/40'
              : 'bg-zinc-900/40 border border-zinc-800/80 hover:border-zinc-700'
          }">
            <div>
              <div class="flex items-start justify-between gap-2 mb-3.5">
                <div class="flex items-center gap-2.5 min-w-0">
                  <div class="w-8 h-8 rounded-full bg-zinc-800 flex items-center justify-center text-xs font-semibold text-zinc-300 border border-zinc-700/60 flex-shrink-0">
                    ${acc.email.substring(0, 2).toUpperCase()}
                  </div>
                  <div class="min-w-0">
                    <h3 class="font-medium text-xs text-zinc-100 truncate" title="${acc.email}">${acc.email}</h3>
                    <p class="text-[10px] text-zinc-500 font-mono mt-0.5 truncate">id: ${acc.id.substring(0, 8)}</p>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  ${
                    isActive
                      ? `<span class="inline-flex items-center gap-1.5 text-[10px] font-medium text-blue-400 px-1 py-0.5">
                          <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
                          Đang dùng
                        </span>`
                      : `<span class="inline-flex items-center gap-1.5 text-[10px] font-medium text-zinc-500 px-1 py-0.5">
                          <span class="w-1.5 h-1.5 rounded-full bg-zinc-600"></span>
                          Sẵn sàng
                        </span>`
                  }
                  <button onclick="deleteAccount('${acc.id}', '${acc.email}')" title="Xóa tài khoản" class="p-1 hover:bg-zinc-800 text-zinc-500 hover:text-red-400 rounded-md transition flex items-center justify-center">
                    <i data-lucide="trash-2" class="w-3.5 h-3.5"></i>
                  </button>
                </div>
              </div>

              <!-- Quota Breakdown -->
              <div class="space-y-2.5 mt-3 pt-3 border-t border-zinc-800/60">
                ${(acc.quota_groups && acc.quota_groups.length > 0) ? acc.quota_groups.map(g => `
                  <div class="space-y-1.5">
                    <div class="text-[11px] font-medium text-zinc-300 flex items-center gap-1">
                      <span>${g.name}</span>
                    </div>
                    <div class="space-y-1.5 bg-zinc-950/40 p-2 rounded-lg border border-zinc-800/50">
                      ${g.buckets.map(b => {
                        let pct = Math.round(b.remaining_percentage);
                        const resetInfo = getResetDisplay(b.reset_time, b.window);
                        if (resetInfo.isExpired) {
                          pct = 100;
                        }
                        const isLow = pct < 20;
                        return `
                        <div>
                          <div class="flex justify-between text-[10px] text-zinc-400 mb-0.5">
                            <span>${b.window === 'FIVE_HOUR' ? 'Hạn ngạch 5 giờ' : (b.window === 'WEEKLY' ? 'Hạn ngạch tuần' : b.window)}</span>
                            <span class="font-mono ${isLow ? 'text-amber-400' : 'text-zinc-200'} font-medium">${pct}%</span>
                          </div>
                          <div class="w-full bg-zinc-900 rounded-full h-1 overflow-hidden border border-zinc-800">
                            <div class="${isLow ? 'bg-amber-400' : 'bg-blue-500'} h-full rounded-full" style="width: ${pct}%"></div>
                          </div>
                          ${resetInfo.text ? `
                            <div class="text-[9px] text-zinc-500 mt-0.5 font-mono">
                              ${resetInfo.text}
                            </div>
                          ` : ''}
                        </div>
                      `}).join('')}
                    </div>
                  </div>
                `).join('') : `
                  <div class="bg-zinc-950/40 p-2 rounded-lg border border-zinc-800/50">
                    <div class="flex justify-between text-[10px] text-zinc-400 mb-0.5">
                      <span>Hạn ngạch khả dụng</span>
                      <span class="font-mono font-medium text-zinc-200">${Math.round(acc.quota_percentage)}%</span>
                    </div>
                    <div class="w-full bg-zinc-900 rounded-full h-1 overflow-hidden border border-zinc-800">
                      <div class="bg-blue-500 h-full rounded-full" style="width: ${Math.round(acc.quota_percentage)}%"></div>
                    </div>
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
                <button onclick="switchAccount('${acc.id}')" class="w-full py-1.5 px-2 text-center text-xs font-medium rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition flex items-center justify-center gap-1.5 cursor-pointer">
                  <i data-lucide="arrow-right-left" class="w-3.5 h-3.5 text-zinc-400"></i> Chuyển sang tài khoản này
                </button>
              `}
            </div>
          </div>
        `;
      }).join('');
      lucide.createIcons();
    }

    async function switchAccount(accountId) {
      try {
        const res = await fetch('/api/accounts/switch', {
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
        const res = await fetch('/api/accounts/delete', {
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
        const res = await fetch('/api/accounts/oauth/start');
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
        const res = await fetch('/api/accounts/add', {
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

    fetchAccounts();
    setInterval(fetchAccounts, 5000);
  </script>
</body>
</html>
"#
}
